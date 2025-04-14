use nostr_sdk::prelude::*;
use std::time::Duration;

use crate::app::Post;

#[derive(Clone)]
pub struct NostrClient {
    //secret key
    client: Client,
    key: Keys
}

impl NostrClient {
    pub fn new(key_str: String) -> Self {
        Self {
            client: Client::new(Keys::parse(&key_str).unwrap()),
            key: Keys::parse(&key_str).unwrap()
        }
        //Self { client, key }
    }
    pub async fn connect_relays(&self, relays: Vec<String>) -> Result<()> {
        for relay in relays {
            self.client.add_relay(relay).await?;
        }
        self.client.connect().await;
        Ok(())
    }

    //This will get who the user is following
    pub async fn fetch_following(&self) -> Vec<PublicKey> {
        let my_pub_key = self.key.public_key();
        let filter = Filter::new().author(my_pub_key).kind(Kind::ContactList);
        let events = self.client.fetch_events(filter, Duration::from_secs(10)).await;
        let mut following_list = vec![];
        if let Some(event) = events.expect("reason").first() {
            let tags = &event.tags;
            
            for tag in tags.iter() {
                if let Some(following) = tag.content() {
                    let following_pk = PublicKey::from_hex(following).unwrap();
                    following_list.push(following_pk);
                }
            }
        }
        following_list
    }

    pub async fn fetch_notes_since(&self,timestamp: Timestamp) -> Result<Vec<Post>> {
        let mut new_posts: Vec<Post> = vec![];

        //IMPORTANT: Update this to get_followers
        let following_list = self.fetch_following().await;
        for pub_key in following_list {
            let filter = Filter::new().author(pub_key).kind(Kind::TextNote)
                .since(Timestamp::now() - timestamp);
            let events = self.client.fetch_events(filter, Duration::from_secs(30)).await?;
            for event in events {
                /*
                let content = &event.content;
                new_posts.push(content.to_string());
                */
                new_posts.push(
                    Post {
                        user: event.pubkey.to_string(),
                        time: event.created_at.to_string(),
                        content: event.content.to_string(),
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
