-- TASK-011-021: Migration Script — Assign Existing Users to Tenants by Email Domain
--
-- Usage:
--   psql -d vaultwarden -f scripts/migrate_to_tenants.sql
--
-- Prerequisites:
--   - Run AFTER the multitenancy migration (2026-04-15-000010_sol_011_multitenancy)
--   - The DEFAULT tenant (00000000-0000-0000-0000-000000000001) must already exist
--   - Run in a transaction for zero-downtime rollback capability
--
-- WARNING: Test on a staging database before running in production!

BEGIN;

-- ─────────────────────────────────────────────────────────────────────────────
-- STEP 1: Verify all existing data has DEFAULT tenant
-- ─────────────────────────────────────────────────────────────────────────────
DO $$
DECLARE
    unassigned_count BIGINT;
BEGIN
    SELECT COUNT(*) INTO unassigned_count
    FROM users
    WHERE tenant_uuid IS NULL OR tenant_uuid = '';

    IF unassigned_count > 0 THEN
        RAISE EXCEPTION 'Found % users without a tenant_uuid. Ensure migration 000010 ran successfully.', unassigned_count;
    END IF;

    RAISE NOTICE 'STEP 1: All users have a tenant_uuid assigned. Proceeding...';
END;
$$;

-- ─────────────────────────────────────────────────────────────────────────────
-- STEP 2: Create per-domain tenants (customize as needed)
-- ─────────────────────────────────────────────────────────────────────────────
-- Example: create tenants for each unique email domain found in users
-- Modify or remove domains you don't need as separate tenants

DO $$
DECLARE
    domain TEXT;
    new_tenant_uuid TEXT;
    domain_slug TEXT;
BEGIN
    FOR domain IN
        SELECT DISTINCT split_part(email, '@', 2) AS domain
        FROM users
        WHERE email LIKE '%@%'
          AND tenant_uuid = '00000000-0000-0000-0000-000000000001' -- only re-assign DEFAULT tenant users
        ORDER BY domain
    LOOP
        -- Skip very short or invalid domains
        IF length(domain) < 3 THEN
            CONTINUE;
        END IF;

        -- Create a URL-safe slug from the domain
        domain_slug := regexp_replace(lower(domain), '[^a-z0-9]', '-', 'g');
        domain_slug := regexp_replace(domain_slug, '-+', '-', 'g');
        domain_slug := btrim(domain_slug, '-');

        -- Skip if tenant for this slug already exists
        IF EXISTS (SELECT 1 FROM tenants WHERE slug = domain_slug) THEN
            RAISE NOTICE 'Tenant for domain % (slug: %) already exists. Skipping.', domain, domain_slug;
            CONTINUE;
        END IF;

        -- Generate UUID
        new_tenant_uuid := gen_random_uuid()::text;

        -- Insert the tenant
        INSERT INTO tenants (uuid, name, slug, domain_restriction, is_active, created_at, updated_at)
        VALUES (
            new_tenant_uuid,
            domain,
            domain_slug,
            domain,
            TRUE,
            NOW(),
            NOW()
        );

        RAISE NOTICE 'Created tenant: % (uuid: %, slug: %) for domain %', domain, new_tenant_uuid, domain_slug, domain;
    END LOOP;
END;
$$;

-- ─────────────────────────────────────────────────────────────────────────────
-- STEP 3: Assign users to tenants based on email domain
-- ─────────────────────────────────────────────────────────────────────────────
DO $$
DECLARE
    tenant_rec RECORD;
    updated_count BIGINT;
BEGIN
    FOR tenant_rec IN
        SELECT uuid, slug, domain_restriction
        FROM tenants
        WHERE domain_restriction IS NOT NULL
          AND uuid != '00000000-0000-0000-0000-000000000001'
    LOOP
        UPDATE users
        SET tenant_uuid = tenant_rec.uuid,
            updated_at = NOW()
        WHERE email LIKE '%@' || tenant_rec.domain_restriction
          AND tenant_uuid = '00000000-0000-0000-0000-000000000001';

        GET DIAGNOSTICS updated_count = ROW_COUNT;
        RAISE NOTICE 'Assigned % users from domain % to tenant %', updated_count, tenant_rec.domain_restriction, tenant_rec.slug;
    END LOOP;
END;
$$;

-- ─────────────────────────────────────────────────────────────────────────────
-- STEP 4: Verification report
-- ─────────────────────────────────────────────────────────────────────────────
SELECT
    t.name AS tenant_name,
    t.slug,
    COUNT(u.uuid) AS user_count
FROM tenants t
LEFT JOIN users u ON u.tenant_uuid = t.uuid
GROUP BY t.uuid, t.name, t.slug
ORDER BY user_count DESC;

-- ─────────────────────────────────────────────────────────────────────────────
-- ROLLBACK PROCEDURE (run these in a separate transaction if needed):
-- ─────────────────────────────────────────────────────────────────────────────
-- UPDATE users SET tenant_uuid = '00000000-0000-0000-0000-000000000001' WHERE tenant_uuid != '00000000-0000-0000-0000-000000000001';
-- DELETE FROM tenants WHERE uuid != '00000000-0000-0000-0000-000000000001';

COMMIT;
