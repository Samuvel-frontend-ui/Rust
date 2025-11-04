-- Your SQL goes here
CREATE TABLE follows (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  target_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  status TEXT NOT NULL DEFAULT 'accepted', -- accepted|pending|rejected
  created_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE (user_id, target_id)
);