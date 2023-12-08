-- Add migration script here
alter table subscriptions
    add column endpoint VARCHAR(2000) unique,
    add column key_p256dh VARCHAR(100),
    add column key_auth VARCHAR(100),
    drop column platform,
    drop column subscription_token;
