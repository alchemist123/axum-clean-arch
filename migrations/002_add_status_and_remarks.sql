-- Add status and remarks to teams table
DO $$ 
BEGIN 
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'team_status') THEN
        CREATE TYPE team_status AS ENUM ('Pending', 'Approved', 'Rejected');
    END IF;
END $$;

ALTER TABLE teams ADD COLUMN IF NOT EXISTS status VARCHAR(20) DEFAULT 'Pending' NOT NULL;
ALTER TABLE teams ADD COLUMN IF NOT EXISTS admin_remarks TEXT;
