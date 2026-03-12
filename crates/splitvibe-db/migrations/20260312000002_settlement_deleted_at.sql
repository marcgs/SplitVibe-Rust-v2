-- Add deleted_at timestamp to settlements for 24h undo window
ALTER TABLE settlements ADD COLUMN deleted_at TIMESTAMPTZ;
