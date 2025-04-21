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
    pub fn new(key_str: String) -> Self {
        Self {
            client: Client::new(Keys::parse(&key_str).unwrap()),
            key: Keys::parse(&key_str).unwrap(),
            contacts: vec![],
        }
    }

    pub fn my_key(&self) -> PublicKey {
        self.key.public_key()
    }

    pub async fn connect_relays(&self, relays: Vec<String>) -> Result<()> {
        for relay in relays {
            self.client.add_relay(relay).await?;
        }
        self.client.connect().await;
        Ok(())
    }

    //This will get who the user is following
    pub async fn fetch_contacts(&self) -> Vec<Contact> {
        let my_pub_key = self.key.public_key();
        let filter = Filter::new().author(my_pub_key).kind(Kind::ContactList);
        let events = self.client.fetch_events(filter, Duration::from_secs(10)).await;
        let mut contacts = vec![];
        if let Some(event) = events.expect("reason").first() {
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

    pub async fn fetch_notes_since(&self,timestamp: Timestamp) -> Result<Vec<Post>> {
        let mut new_posts: Vec<Post> = vec![];

        //let following_list = &self.contacts;
        for contact in self.contacts.clone() {
            let pub_key = contact.key;
            let user = contact.name;
            let filter = Filter::new()
                .author(pub_key)
                .kind(Kind::TextNote)
                .since(timestamp);
            let events = self.client.fetch_events(filter, Duration::from_secs(30)).await?;
            for event in events {
                let utc_time = Utc.timestamp_opt(event.created_at.as_u64() as i64,0).unwrap();
                let local_time: DateTime<Local> = DateTime::from(utc_time);
                let datetime = local_time.format("%H:%M %h-%d-%Y").to_string();

                new_posts.push(
                    Post {
                        user: user.clone(),
                        timestamp: event.created_at.as_u64(),
                        datetime: datetime,
                        content: event.content.to_string(),
                        id: event.id.to_hex(),
                    }
                );
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
