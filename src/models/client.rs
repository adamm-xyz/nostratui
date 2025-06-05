use std::time::Duration;
use chrono::{DateTime, Local, Utc, TimeZone};
use nostr_sdk::prelude::*;
use crate::models::post::Post;
use crate::error::NostratuiError;
use anyhow::{Context, Result};
use tokio::time::timeout;

#[derive(Clone)]
pub struct NostrClient {
    //secret key
    client: Client,
    key: Keys,
    contacts: Vec<Contact>,
    relays: Vec<String>
}


#[derive(Clone)]
pub struct Contact {
    key: PublicKey,
    name: String,
}

impl Contact {
    pub fn to_string_tuple(&self) -> (String,String) {
        (self.key.to_bech32().unwrap(),self.name.clone())
    }
}

impl NostrClient {
    pub fn new(key_str: String) -> Result<Self> {
        let key = Keys::parse(&key_str)
            .map_err(|e| NostratuiError::KeyParsing(e.to_string()))?;

        Ok( Self {
            client: Client::new(key.clone()),
            key,
            contacts: vec![],
            relays: vec![],
        })
    }

    pub fn get_contacts(&self) -> Vec<Contact> {
        return self.contacts.clone()
    }

    pub fn print(&self) -> Result<()> {
        println!(
            "Key: {}\n Num of contacts: {}\n",
            self.key.public_key().to_bech32().unwrap(),
            self.contacts.len());
            Ok(())
    }

    pub fn my_key(&self) -> PublicKey {
        self.key.public_key()
    }

    pub fn set_relays(&mut self, relays: Vec<String>) {
        self.relays = relays;
    }

    pub fn get_relays(&self) -> Vec<String> {
        return self.relays.clone()
    }

    pub async fn connect_relays(&mut self) -> Result<(),NostratuiError> {
        let mut connection_results = Vec::new();

        for relay in &self.relays {
            match self.client.add_relay(relay.clone()).await {
                Ok(_) => connection_results.push(Ok(())),
                Err(e) => connection_results.push(Err(NostratuiError::Network(
                            format!("Failed to add relay {}: {}", relay, e)
                ))),
            }
        }

        self.client.connect().await;

        if connection_results.iter().any(Result::is_ok) {
            Ok(())
        } else {
            Err(NostratuiError::Network("Failed to connect to any relays".to_string()).into())
        }
        
    }

    //This will get who the user is following
    pub async fn fetch_contacts(&mut self) -> Result<Vec<Contact>> {
        let my_pub_key = self.key.public_key();
        let filter = Filter::new().author(my_pub_key).kind(Kind::ContactList);
        
        // Use timeout for fetching events
        let my_contacts_note = timeout(
            Duration::from_secs(15),
            self.client.fetch_events(filter, Duration::from_secs(10))
        )
        .await
        .context("Timeout fetching contact list")?
        .context("Failed to fetch contact list")?;
        
        let mut contacts = vec![];
        
        if let Some(fetched_contact_list) = my_contacts_note.first() {
            let tags = &fetched_contact_list.tags;
            
            
            for tag in tags.iter() {
                if let Some(following) = tag.content() {
                    let following_pk = match PublicKey::from_hex(following) {
                        Ok(pk) => pk,
                        Err(_) => continue,
                    };
                    
                    let metadata_filter = Filter::new()
                        .author(following_pk)
                        .kind(Kind::Metadata);
                            
                    let metadata_result = self.client.fetch_events(metadata_filter, Duration::from_secs(10)).await;
                        
                    match metadata_result {
                        Ok(fetched_contact_metadata) => {
                            if let Some(metadata) = fetched_contact_metadata.first() {
                                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&metadata.content) {
                                    if let Some(name) = value.get("name").and_then(|n| n.as_str()) {
                                        println!("found metadata for name!");
                                        contacts.push(
                                            Contact {
                                                key: following_pk,
                                                name: name.to_string(),
                                            });
                                    }                                 }
                            } else {
                                println!("Needed fallback");
                                // Fallback to using pubkey as name
                                contacts.push(
                                    Contact {
                                        key: following_pk,
                                        name: following_pk.to_bech32().unwrap_or_default(),
                                    });
                            }
                        },
                        _ => {}

                    }
                }
            }
        }
            
        
        Ok(contacts)
    }

    pub async fn set_contacts(&mut self, contacts: Vec<(String,String)>) -> Result<()> {
        if contacts.is_empty() {
            self.contacts = self.fetch_contacts().await?;
        } else {
            let mut contact_list = vec![];
            for contact in contacts {
                let (contact_key,contact_name) = contact;
                contact_list.push(
                    Contact {
                        key: PublicKey::parse(&contact_key).unwrap(),
                        name: contact_name
                    }
                )
            }
            self.contacts = contact_list;
        }
        Ok(())
    }

    pub async fn fetch_notes_since(&self, timestamp: Timestamp) -> Result<Vec<Post>, NostratuiError> {
        let mut new_posts: Vec<Post> = vec![];
        let mut tasks = Vec::new();
        
        for contact in self.contacts.clone() {
            let client_clone = self.client.clone();
            let pub_key = contact.key;
            let user = contact.name.clone();
            
            tasks.push(tokio::spawn(async move {
                let filter = Filter::new()
                    .author(pub_key)
                    .kind(Kind::TextNote)
                    .since(timestamp);
                    
                let result = tokio::time::timeout(
                    Duration::from_secs(10),
                    client_clone.fetch_events(filter, Duration::from_secs(30))
                ).await;
                
                match result {
                    Ok(Ok(events)) => {
                        let mut contact_posts = Vec::new();
                        for event in events {
                            let utc_time = Utc.timestamp_opt(event.created_at.as_u64() as i64, 0).unwrap();
                            let local_time: DateTime<Local> = DateTime::from(utc_time);
                            let datetime = local_time.format("%H:%M %h-%d-%Y").to_string();
                            
                            // Extract thread information from tags
                            let mut root_id = None;
                            let mut reply_id = None;
                            let mut mentions = Vec::new();
                            let mut participants = Vec::new();

                            // Process tags for thread information
                            for tag in event.tags.iter() {
                                let tag = (*tag).clone();
                                let vec = tag.to_vec();
                                if vec.len() >= 2 {
                                    match vec[0].as_str() {
                                        "e" => {
                                            let event_id = vec[1].clone();
                                            if vec.len() >= 4 {
                                                match vec[3].as_str() {
                                                    "root" => root_id = Some(event_id),
                                                    "reply" => reply_id = Some(event_id),
                                                    _ => mentions.push(event_id),
                                                }
                                            } else {
                                                // Handle deprecated positional e tags
                                                if vec.len() == 2 {
                                                    reply_id = Some(event_id);
                                                } else if vec.len() == 3 {
                                                    root_id = Some(event_id);
                                                }
                                            }
                                        }
                                        "p" => {
                                            participants.push(vec[1].clone());
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            
                            contact_posts.push(Post {
                                user: user.clone(),
                                timestamp: event.created_at.as_u64(),
                                datetime,
                                content: event.content.to_string(),
                                id: event.id.to_hex(),
                                root_id,
                                reply_id,
                                mentions,
                                participants,
                            });
                        }
                        Ok(contact_posts)
                    }
                    Ok(Err(e)) => Err(NostratuiError::NostrSdk(e.to_string())),
                    Err(_) => Err(NostratuiError::NostrSdk("Timeout fetching events".to_string())),
                }
            }));
        }
        
        for task in futures::future::join_all(tasks).await {
            match task {
                Ok(Ok(posts)) => new_posts.extend(posts),
                Ok(Err(e)) => log::warn!("Error fetching posts: {}", e),
                Err(e) => log::warn!("Task error: {}", e),
            }
        }
        
        Ok(new_posts)
    }

    pub async fn post_note(&self, note: String, reply_to: Option<(String, String)>) -> Result<(),NostratuiError> {
        let mut builder = EventBuilder::text_note(note).pow(20);
        
        // If this is a reply, add the appropriate e tags
        if let Some((root_id, reply_id)) = reply_to {
            // Add root tag
            let root_tag = Tag::parse(["e", &root_id, "", "root"])
                .map_err(|e| NostratuiError::NostrSdk(e.to_string()))?;
            builder = builder.tag(root_tag);
            
            // Add reply tag if it's different from root
            if reply_id != root_id {
                let reply_tag = Tag::parse(["e", &reply_id, "", "reply"])
                    .map_err(|e| NostratuiError::NostrSdk(e.to_string()))?;
                builder = builder.tag(reply_tag);
            }
        }
            
        self.client.send_event_builder(builder).await?;
        Ok(())
    }

    pub async fn fetch_thread(&self, root_id: &str) -> Result<Vec<Post>, NostratuiError> {
        let root_event_id = EventId::from_hex(root_id)
            .map_err(|e| NostratuiError::NostrSdk(e.to_string()))?;

        // Create a filter to get all posts in the thread
        let filter = Filter::new()
            .kind(Kind::TextNote)
            .event(root_event_id);

        let events = self.client.fetch_events(filter, Duration::from_secs(30)).await?;
        
        // Convert events to posts and sort by timestamp
        let mut posts: Vec<Post> = events.into_iter()
            .map(|event| {
                let root_id = event.tags.iter()
                    .find(|t| {
                        let tag = (*t).clone();
                        let vec = tag.to_vec();
                        vec.get(0) == Some(&"e".to_string()) && vec.get(3) == Some(&"root".to_string())
                    })
                    .and_then(|t| {
                        let tag = (*t).clone();
                        tag.to_vec().get(1).cloned()
                    });

                let reply_id = event.tags.iter()
                    .find(|t| {
                        let tag = (*t).clone();
                        let vec = tag.to_vec();
                        vec.get(0) == Some(&"e".to_string()) && vec.get(3) == Some(&"reply".to_string())
                    })
                    .and_then(|t| {
                        let tag = (*t).clone();
                        tag.to_vec().get(1).cloned()
                    });

                let mentions = event.tags.iter()
                    .filter(|t| {
                        let tag = (*t).clone();
                        let vec = tag.to_vec();
                        vec.get(0) == Some(&"e".to_string()) && 
                        vec.get(3) != Some(&"root".to_string()) && 
                        vec.get(3) != Some(&"reply".to_string())
                    })
                    .filter_map(|t| {
                        let tag = (*t).clone();
                        tag.to_vec().get(1).cloned()
                    })
                    .collect();

                let participants = event.tags.iter()
                    .filter(|t| {
                        let tag = (*t).clone();
                        let vec = tag.to_vec();
                        vec.get(0) == Some(&"p".to_string())
                    })
                    .filter_map(|t| {
                        let tag = (*t).clone();
                        tag.to_vec().get(1).cloned()
                    })
                    .collect();

                Post {
                    id: event.id.to_hex(),
                    user: event.pubkey.to_hex(),
                    content: event.content,
                    timestamp: event.created_at.as_u64(),
                    datetime: event.created_at.to_human_datetime(),
                    root_id,
                    reply_id,
                    mentions,
                    participants,
                }
            })
            .collect();
        
        posts.sort_by_key(|post| post.timestamp);
        
        Ok(posts)
    }

}
