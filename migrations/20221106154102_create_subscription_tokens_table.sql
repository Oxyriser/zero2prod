CREATE TABLE subscription_token (
  subscription_token TEXT PRIMARY KEY,
  subscriber_id uuid NOT NULL REFERENCES subscription(id)
);
