-- Productivity: time tracking sessions (null end_time = active)
CREATE TABLE IF NOT EXISTS productivity_time_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_name TEXT NOT NULL,
    jira_id TEXT,
    project_key TEXT,
    work_type VARCHAR(20) NOT NULL CHECK (work_type IN ('office', 'personal')),
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ,
    duration_ms BIGINT
);

CREATE INDEX IF NOT EXISTS idx_productivity_time_logs_end_time
    ON productivity_time_logs (end_time);
CREATE INDEX IF NOT EXISTS idx_productivity_time_logs_start_time
    ON productivity_time_logs (start_time);
CREATE INDEX IF NOT EXISTS idx_productivity_time_logs_office_ended
    ON productivity_time_logs (work_type, end_time) WHERE work_type = 'office';

-- At most one active (unstopped) session
CREATE UNIQUE INDEX IF NOT EXISTS productivity_one_active_time_log
    ON productivity_time_logs ((1))
    WHERE end_time IS NULL;

-- Planning queue
CREATE TABLE IF NOT EXISTS productivity_queue_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    text TEXT NOT NULL,
    done BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    position INT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_productivity_queue_position ON productivity_queue_items (position);

-- Schedule events
CREATE TABLE IF NOT EXISTS productivity_schedule_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    event_date DATE NOT NULL,
    time_str VARCHAR(64) NOT NULL,
    event_type VARCHAR(64) NOT NULL DEFAULT 'Video'
);

CREATE INDEX IF NOT EXISTS idx_productivity_schedule_event_date
    ON productivity_schedule_events (event_date);

-- Single-row layout
CREATE TABLE IF NOT EXISTS productivity_layout (
    id SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    layout_order JSONB NOT NULL DEFAULT '[]'::jsonb,
    layout_sizes JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Single-row goals / settings
CREATE TABLE IF NOT EXISTS productivity_settings (
    id SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    tasks_goal INT NOT NULL DEFAULT 14,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO productivity_settings (id, tasks_goal)
VALUES (1, 14)
ON CONFLICT (id) DO NOTHING;

INSERT INTO productivity_layout (id, layout_order, layout_sizes)
VALUES (
    1,
    '["clock", "task-logger", "stats", "calendar", "planning", "report"]'::jsonb,
    '{}'::jsonb
)
ON CONFLICT (id) DO NOTHING;
