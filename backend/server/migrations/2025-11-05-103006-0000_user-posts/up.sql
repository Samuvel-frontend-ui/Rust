
CREATE TABLE user_posts (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  description TEXT NOT NULL,
  videos TEXT[] NOT NULL,
  created_at TIMESTAMP DEFAULT NOW()
);

