-- TASK-011-011: PostgreSQL Row-Level Security for Multi-Tenancy
-- Only applied when TENANT_RLS_ENABLED=true and using PostgreSQL backend

-- Function to set the current tenant context for the session
CREATE OR REPLACE FUNCTION set_current_tenant(tenant_uuid TEXT)
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.current_tenant', tenant_uuid, TRUE);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Helper function to read the current tenant
CREATE OR REPLACE FUNCTION current_tenant()
RETURNS TEXT AS $$
BEGIN
    RETURN COALESCE(current_setting('app.current_tenant', TRUE), '');
END;
$$ LANGUAGE plpgsql STABLE;

-- ─────────────────────────────────────────────────────────────────────────────
-- Enable RLS on users
-- ─────────────────────────────────────────────────────────────────────────────
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE users FORCE ROW LEVEL SECURITY;

-- Allow access to rows matching the current tenant (or SYSTEM_ADMIN sentinel bypasses)
CREATE POLICY tenant_isolation_users
    ON users
    AS PERMISSIVE
    FOR ALL
    USING (
        tenant_uuid = current_tenant()
        OR current_tenant() = 'SYSTEM_ADMIN'
    )
    WITH CHECK (
        tenant_uuid = current_tenant()
        OR current_tenant() = 'SYSTEM_ADMIN'
    );

-- ─────────────────────────────────────────────────────────────────────────────
-- Enable RLS on organizations
-- ─────────────────────────────────────────────────────────────────────────────
ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE organizations FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_organizations
    ON organizations
    AS PERMISSIVE
    FOR ALL
    USING (
        tenant_uuid = current_tenant()
        OR current_tenant() = 'SYSTEM_ADMIN'
    )
    WITH CHECK (
        tenant_uuid = current_tenant()
        OR current_tenant() = 'SYSTEM_ADMIN'
    );

-- ─────────────────────────────────────────────────────────────────────────────
-- Enable RLS on audit_entries
-- ─────────────────────────────────────────────────────────────────────────────
ALTER TABLE audit_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_entries FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_audit_entries
    ON audit_entries
    AS PERMISSIVE
    FOR ALL
    USING (
        tenant_uuid = current_tenant()
        OR current_tenant() = 'SYSTEM_ADMIN'
    )
    WITH CHECK (
        tenant_uuid = current_tenant()
        OR current_tenant() = 'SYSTEM_ADMIN'
    );
