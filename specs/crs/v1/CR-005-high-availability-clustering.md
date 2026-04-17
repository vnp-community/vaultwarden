# CR-005: High Availability & Horizontal Scaling Architecture

> **Change Request ID**: CR-005  
> **Title**: High Availability, Horizontal Scaling & Clustering Support  
> **Priority**: P1 — Critical  
> **Target Release**: v2.0  
> **Driven By**: [specs/crs/product-market-analysis.md §2.3]  
> **Affects**: PRD §9.3, URD §6, SRS §5.3, Technical Design §2

---

## 1. Problem Statement

Kiến trúc hiện tại là **single-instance**:
- Single Vaultwarden process là Single Point of Failure (SPOF)
- WebSocket state (`DashMap`) là in-memory, không shared giữa nhiều instances
- Không thể horizontal scale — tất cả connections phải đến cùng một process
- Banking SLA yêu cầu **99.99% uptime** (52 phút downtime/năm) — single-instance không đạt được
- Planned maintenance (update, restart) gây downtime

---

## 2. Scope of Change

### 2.1 Stateless HTTP Layer

Mục tiêu: mọi HTTP request có thể được xử lý bởi bất kỳ instance nào trong cluster.

**Loại bỏ shared in-memory state**:
- JWT validation: đã stateless (verify signature)
- Rate limiter (`LIMITER_LOGIN`): chuyển sang Redis
- WebSocket connections: chuyển sang Redis pub/sub
- OIDC state cache: chuyển sang Redis
- 2FA remember tokens: verify against DB (không in-memory)

### 2.2 Redis-Backed Shared State

```
NEW CONFIG:
REDIS_URL=redis://redis-cluster:6379
REDIS_TLS=true
REDIS_PASSWORD=<secret>
REDIS_POOL_SIZE=20
REDIS_KEY_PREFIX=vaultwarden:
REDIS_ENABLED=false                    # Default: false (single-instance mode)
```

**Redis usage**:

| State | Redis Key Pattern | TTL |
|-------|------------------|-----|
| Rate limiter (login) | `vw:rl:login:{ip}` | 60s |
| Rate limiter (admin) | `vw:rl:admin:{ip}` | 60s |
| OIDC state cache | `vw:oidc:{state}` | 10min |
| Duo contexts | `vw:duo:{ctx}` | 15min |
| WebSocket event bus | `vw:ws:events` | N/A (pub/sub) |
| Auth request cache | `vw:authreq:{id}` | 5min |
| 2FA incomplete | `vw:2fa_incomplete:{user}` | 1h |

### 2.3 WebSocket Architecture — Redis Pub/Sub

```
┌──────────────┐        ┌──────────────┐
│ Instance A   │        │ Instance B   │
│              │        │              │
│ User X WS ←─┼──────┐ │ User Y WS ←─┤
│              │      │ │              │
│ Local DashMap│      │ │ Local DashMap│
└──────┬───────┘      │ └──────┬───────┘
       │               │        │
       │  publish event │        │
       ↓               └────────┘
┌──────────────────────────────────────┐
│          Redis Pub/Sub               │
│   Channel: vw:ws:user:{user_uuid}    │
└──────────────────────────────────────┘
```

**Flow**:
1. Event occurs on Instance A (e.g., cipher updated)
2. Instance A publishes event to Redis channel `vw:ws:user:{user_uuid}`
3. All instances subscribed to that channel receive event
4. Each instance forwards event to its local WebSocket connections for that user

### 2.4 Database High Availability

**PostgreSQL** (recommended for HA):
- Read replica support: `DATABASE_READ_URL` for read-heavy queries (sync, list)
- Connection pool health checks
- Automatic failover to replica on primary failure (via connection string or pgBouncer)

**Config**:
```
NEW CONFIG:
DATABASE_READ_URL=postgresql://replica:5432/vaultwarden    # Optional read replica
DATABASE_READ_POOL_SIZE=10
DATABASE_WRITE_URL=postgresql://primary:5432/vaultwarden   # Explicit write URL
```

### 2.5 Deployment Architecture

```
                    ┌─────────────────────────────────────┐
                    │         Load Balancer                │
                    │    (nginx / HAProxy / AWS ALB)       │
                    └─────────────┬───────────────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
    ┌─────────▼───────┐ ┌─────────▼───────┐ ┌────────▼────────┐
    │  Vaultwarden    │ │  Vaultwarden    │ │  Vaultwarden    │
    │  Instance A     │ │  Instance B     │ │  Instance C     │
    └─────────┬───────┘ └─────────┬───────┘ └────────┬────────┘
              │                   │                   │
              └───────────────────┼───────────────────┘
                                  │
                    ┌─────────────┴────────────┐
                    │                          │
          ┌─────────▼──────┐       ┌──────────▼──────┐
          │   PostgreSQL   │       │     Redis        │
          │   Primary      │       │   Cluster        │
          │   + Replicas   │       │                  │
          └────────────────┘       └─────────────────-┘
```

### 2.6 Session Affinity for WebSocket

WebSocket connections cần sticky session (nhưng không required nếu dùng Redis pub/sub):
- Load balancer: `ip_hash` hoặc cookie-based sticky session
- Fallback: Redis pub/sub ensures all instances forward events regardless

### 2.7 Zero-Downtime Deployment

- **Health check endpoint**: `GET /health` → returns `{"status":"ok","version":"x.y.z","db":"ok","redis":"ok"}`
- **Graceful shutdown**: drain existing connections, stop accepting new ones, shutdown after `SHUTDOWN_TIMEOUT_SECONDS`
- **Rolling update support**: Load balancer removes instance from pool before restart
- **Migration safety**: Migrations run once (distributed lock via DB) on startup

```
NEW CONFIG:
SHUTDOWN_TIMEOUT_SECONDS=30
INSTANCE_ID=auto                       # Auto-generated or manually set
CLUSTER_MODE=false                     # Enable cluster mode (requires Redis)
```

---

## 3. Acceptance Criteria

- [ ] Three instances behind load balancer: killing one instance does not cause user-visible errors
- [ ] WebSocket event on Instance A delivered to user connected on Instance B via Redis pub/sub
- [ ] Rate limiter state shared between instances (login blocked on Instance B after limit hit on Instance A)
- [ ] `GET /health` returns 200 with all subsystems healthy
- [ ] Rolling update: new instance passes health check before traffic is routed to it
- [ ] Zero downtime during planned restart (graceful shutdown timeout honored)
- [ ] OIDC flow works when Instance A generates nonce and Instance B handles callback

---

## 4. Migration Path

1. **Phase 1**: Add Redis support (optional), no breaking changes
2. **Phase 2**: Externalize all in-memory state to Redis when `CLUSTER_MODE=true`
3. **Phase 3**: Test with 3-instance cluster; document Kubernetes Helm chart
4. **Phase 4**: PostgreSQL read replica support

---

## 5. Estimated Effort

| Area | Effort |
|------|--------|
| Redis client integration | 1 sprint |
| Rate limiter → Redis | 1 sprint |
| OIDC state → Redis | 0.5 sprint |
| WebSocket Redis pub/sub | 3 sprints |
| Health check endpoint | 0.5 sprint |
| DB read replica support | 1 sprint |
| Graceful shutdown | 1 sprint |
| Kubernetes Helm chart | 2 sprints |
| Load testing (3-node cluster) | 1 sprint |

---

*Status: Draft | Author: Product Team | Date: 2026-04-12*
