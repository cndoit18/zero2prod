mod subscriber_email;
mod subscriber_name;

pub use subscriber_email::*;
pub use subscriber_name::*;

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
