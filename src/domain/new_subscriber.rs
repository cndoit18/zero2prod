use super::subscriber_email::*;
use super::subscriber_name::*;

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
