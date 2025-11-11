-- Initialize hodei-server database
-- This script runs when TimescaleDB container starts for the first time

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

-- Create admin user (optional, for development)
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'hodei_admin') THEN
        CREATE ROLE hodei_admin WITH LOGIN PASSWORD 'admin' SUPERUSER;
    END IF;
END
$$;

-- Create extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Set default permissions
GRANT ALL PRIVILEGES ON DATABASE hodei_db TO hodei;
GRANT ALL PRIVILEGES ON SCHEMA public TO hodei;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO hodei;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO hodei;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO hodei;

-- Create default project for testing
INSERT INTO projects (id, name, description, default_branch)
VALUES ('test-project', 'Test Project', 'Test project for hodei-server', 'main')
ON CONFLICT (id) DO NOTHING;

-- Create default admin user (for testing - change password in production!)
DO $$
DECLARE
    user_id uuid := uuid_generate_v4();
BEGIN
    -- Insert admin user (password: admin123 - change this!)
    -- In production, use proper password hashing
    IF NOT EXISTS (SELECT FROM users WHERE username = 'admin') THEN
        INSERT INTO users (id, username, email, password_hash, role)
        VALUES (
            user_id,
            'admin',
            'admin@hodei.local',
            '$2b$10$example.hash.for.admin123',
            'admin'
        );
    END IF;
END
$$;

-- Create a view for finding statistics
CREATE OR REPLACE VIEW finding_statistics AS
SELECT 
    p.id as project_id,
    p.name as project_name,
    COUNT(DISTINCT a.id) as total_analyses,
    COUNT(DISTINCT f.id) as total_findings,
    COUNT(DISTINCT CASE WHEN f.severity = 'critical' THEN f.id END) as critical_findings,
    COUNT(DISTINCT CASE WHEN f.severity = 'major' THEN f.id END) as major_findings,
    COUNT(DISTINCT CASE WHEN f.severity = 'minor' THEN f.id END) as minor_findings,
    COUNT(DISTINCT CASE WHEN f.severity = 'info' THEN f.id END) as info_findings,
    MIN(a.timestamp) as first_analysis,
    MAX(a.timestamp) as last_analysis
FROM projects p
LEFT JOIN analyses a ON a.project_id = p.id
LEFT JOIN findings f ON f.analysis_id = a.id
GROUP BY p.id, p.name;

-- Grant permissions on views
GRANT SELECT ON finding_statistics TO hodei;

-- Print initialization message
DO $$
BEGIN
    RAISE NOTICE 'hodei-server database initialized successfully!';
    RAISE NOTICE 'Default admin user: admin@hodei.local';
    RAISE NOTICE 'Default password: admin123 (CHANGE IN PRODUCTION!)';
    RAISE NOTICE 'Test project created with ID: test-project';
END
$$;
