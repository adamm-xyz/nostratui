use nostr_sdk::prelude::*;
use std::time::Duration;

use crate::app::Post;

use chrono::{DateTime, TimeZone, Local, Utc};

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
        let key = Keys::parse(&key_str).unwrap();
        Ok( Self {
            client: Client::new(key.clone()),
            key,
            contacts: vec![],
            relays: vec![],
        })
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

    pub async fn connect_relays(&mut self, relays: Vec<String>) -> Result<bool> {
        // Add all relays first
        for relay in &relays {
            self.client.add_relay(relay.clone()).await?;
            self.relays.push(relay.clone());
        }
        
        // Connect to network
        self.client.connect().await;
        // Wait a moment for connections to establish
        tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;
        
        // Get relay connections and check their status
        let relay_connections = self.client.relays();
        let connected = relay_connections.await.iter().any(|(_, relay)| {
            relay.status() == RelayStatus::Connected
        });
        
        Ok(connected)
        
    }

    pub async fn is_connected(&mut self) -> Result<bool> {
        return self.connect_relays(self.relays.clone()).await;
    }

    //This will get who the user is following
    pub async fn fetch_contacts(&mut self) -> Vec<Contact> {
        let my_pub_key = self.key.public_key();
        println!("my pubkey {}",my_pub_key);
        let filter = Filter::new().author(my_pub_key).kind(Kind::ContactList);
        let events = match self.client.fetch_events(filter, Duration::from_secs(30)).await {
            Ok(evts) => evts,
            Err(e) => {
                println!("Error fetching ContactList event: {:?}", e);
                return vec![];
            }
        };

        if events.is_empty() {
            println!("No ContactList events found for your public key");
            return vec![];
        }
        let mut contacts = vec![];
        if let Some(event) = events.first() {
            println!("received contact list");
            let tags = &event.tags;
            
            for tag in tags.iter() {
                if let Some(following) = tag.content() {
                    let following_pk = PublicKey::from_hex(following).unwrap();
                    let metadata_filter = Filter::new().author(following_pk).kind(Kind::Metadata);
                    let metadata_events = self.client.fetch_events(metadata_filter, Duration::from_secs(10)).await;
                    if let Some(metadata) = metadata_events.expect("reason").first() {
                        let data = &metadata.content;
                        let v: Value = serde_json::from_str(data).unwrap();
                        // Extract the "name" value
                        if let Some(name) = v.get("name") {
                            if let Some(name_str) = name.as_str() {
                                println!("{}",name_str);
                                contacts.push(
                                    Contact {
                                        key: following_pk,
                                        name: name_str.to_string()
                                    }
                                )
                            }
                       }
                   } else {
                       println!("No metadata found for {}", following_pk);
                       contacts.push(
                            Contact {
                                key: following_pk,
                                name: following_pk.to_bech32().unwrap()
                            }
                       )
                   }
                }
            }
        }
        contacts
    }

    pub async fn set_contacts(&mut self, contacts: Vec<(String,String)>) -> Result<()> {
        if contacts.is_empty() {
            self.contacts = self.fetch_contacts().await;
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

    pub async fn fetch_notes_since(&self, timestamp: Timestamp) -> Result<Vec<Post>> {
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
                            
                            contact_posts.push(Post {
                                user: user.clone(),
                                timestamp: event.created_at.as_u64(),
                                datetime,
                                content: event.content.to_string(),
                                id: event.id.to_hex(),
                            });
                        }
                        Ok(contact_posts)
                    }
                    Ok(Err(e)) => Err(anyhow::anyhow!("Failed to fetch events: {}", e)),
                    Err(_) => Err(anyhow::anyhow!("Timeout fetching events")),
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


    pub async fn post_note(&self, note: String) -> Result<()> {
        let builder = EventBuilder::text_note(note).pow(20);
        self.client.send_event_builder(builder).await?;
        Ok(())
    }

}
