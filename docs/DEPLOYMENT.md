# IdeaForge Self-Hosted Deployment Guide

Deploy IdeaForge on a Hetzner VPS alongside existing tenants (Discourse forum, StoneWalker Django app, simplex-rss-bot). This guide is a single top-to-bottom walkthrough — everything you need is right here.

**Target setup**: Ubuntu 22.04+, shared Hetzner box, nginx reverse proxy, systemd-managed Axum binary, dedicated PostgreSQL instance/DB, Let's Encrypt SSL. Build happens in GitHub Actions, not on the server.

**Key design decisions**:
- **Build in CI, ship artifacts.** A `cargo build --release` of the IdeaForge workspace peaks at 3–6 GB RAM. Building on a shared server risks OOM-killing Discourse's Unicorn/Sidekiq workers. CI builds the binary + the Trunk `dist/` bundle and scps them over.
- **Atomic releases via symlink swap.** Trunk embeds SRI hashes in `index.html` pointing at specific hashed WASM/CSS filenames. A partial rsync into a live `dist/` would produce hash/asset skew (browser blocks stylesheet with integrity mismatch). Each release lands in `releases/<timestamp>-<sha>/` and `current` is atomically re-pointed.
- **Resource-capped systemd unit.** `MemoryMax` and `CPUQuota` ensure a runaway Axum process can't starve co-tenants.
- **Dedicated Postgres DB/role.** Don't share with Discourse (different version/extension expectations). Reuse the existing cluster with a new DB + role, or run a separate instance on another port.

---

## 0. Point the Subdomain at the Server

Pick a subdomain (e.g. `ideaforge.yourdomain.org`) so the existing Discourse/StoneWalker nginx vhosts stay untouched.

At your DNS registrar:

```
Type: A
Name: ideaforge
Value: YOUR_SERVER_IPV4
TTL: 300
```

```
Type: AAAA
Name: ideaforge
Value: YOUR_SERVER_IPV6
TTL: 300
```

Verify propagation at [dnschecker.org](https://dnschecker.org) before running certbot in step 8.

---

## 1. System Setup

SSH in as root (or with sudo):

```bash
ssh root@YOUR_SERVER_IP
```

```bash
apt update && apt upgrade -y

# Runtime only — NO rustup, NO build-essential for the app itself.
# (build-essential may already be installed for Discourse; that's fine.)
apt install -y nginx certbot python3-certbot-nginx git curl ca-certificates \
    postgresql-client

# Application user — no shell login, only for running the binary
useradd --system --shell /usr/sbin/nologin --home /opt/ideaforge ideaforge
mkdir -p /opt/ideaforge/{releases,shared,bin}
chown -R ideaforge:ideaforge /opt/ideaforge

# Deploy user — used by GitHub Actions to scp artifacts and run deploy.sh
useradd --system --shell /bin/bash --home /home/deploy deploy
mkdir -p /home/deploy/.ssh
chmod 700 /home/deploy/.ssh
# Paste the CI deploy public key here:
# echo "ssh-ed25519 AAAA... github-actions-ideaforge" >> /home/deploy/.ssh/authorized_keys
chown -R deploy:deploy /home/deploy/.ssh
chmod 600 /home/deploy/.ssh/authorized_keys
```

Grant the `deploy` user exactly the two sudoers privileges it needs (nothing more):

```bash
cat > /etc/sudoers.d/ideaforge-deploy <<'EOF'
deploy ALL=(root) NOPASSWD: /bin/systemctl restart ideaforge
deploy ALL=(root) NOPASSWD: /bin/systemctl reload nginx
EOF
chmod 440 /etc/sudoers.d/ideaforge-deploy
```

Layout:

```
/opt/ideaforge/
├── bin/                    # symlink target for the current binary
│   └── ideaforge -> ../current/ideaforge
├── current -> releases/2026-04-15T12-00-00-abc123/
├── releases/
│   ├── 2026-04-15T12-00-00-abc123/
│   │   ├── ideaforge       # Axum binary
│   │   └── dist/           # Trunk output (index.html + hashed wasm/css)
│   └── 2026-04-14T09-30-00-def456/   # previous release, kept for rollback
└── shared/
    └── .env                # secrets — not in git, not overwritten by deploys
```

---

## 2. PostgreSQL Setup

Reuse the existing cluster (the one Discourse/StoneWalker already run against) with a new role + database:

```bash
sudo -u postgres psql <<'EOF'
CREATE ROLE ideaforge WITH LOGIN PASSWORD 'REPLACE_WITH_STRONG_PASSWORD';
CREATE DATABASE ideaforge OWNER ideaforge;
GRANT ALL PRIVILEGES ON DATABASE ideaforge TO ideaforge;
EOF
```

If you'd rather fully isolate IdeaForge (recommended long-term), run a second Postgres cluster on port 5433 via `pg_createcluster 16 ideaforge --port=5433` — but the shared-cluster approach is fine for MVP.

Note the connection string for the `.env` below:

```
postgres://ideaforge:REPLACE_WITH_STRONG_PASSWORD@localhost:5432/ideaforge?sslmode=prefer
```

---

## 3. Environment File

Create `/opt/ideaforge/shared/.env` (owned by `ideaforge:ideaforge`, mode `600`):

```bash
sudo -u ideaforge tee /opt/ideaforge/shared/.env > /dev/null <<'EOF'
# --- Core ---
RUST_LOG=info
BIND_ADDR=127.0.0.1:8080
PUBLIC_URL=https://ideaforge.yourdomain.org

# --- Database ---
DATABASE_URL=postgres://ideaforge:REPLACE_WITH_STRONG_PASSWORD@localhost:5432/ideaforge?sslmode=prefer

# --- Auth / JWT ---
JWT_SECRET=REPLACE_WITH_64_BYTES_HEX
ARGON2_SECRET=REPLACE_WITH_64_BYTES_HEX

# --- Stripe (fiat payments) ---
STRIPE_SECRET_KEY=
STRIPE_WEBHOOK_SECRET=

# --- Cardano / Blockfrost ---
BLOCKFROST_API_KEY=
CARDANO_NETWORK=preprod

# --- NATS ---
NATS_URL=nats://localhost:4222

# --- Frontend dist path (served by nginx) ---
FRONTEND_DIST=/opt/ideaforge/current/dist
EOF
chmod 600 /opt/ideaforge/shared/.env
```

Generate strong secrets with:

```bash
openssl rand -hex 64
```

---

## 4. systemd Unit

Create `/etc/systemd/system/ideaforge.service`:

```ini
[Unit]
Description=IdeaForge Axum API
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=ideaforge
Group=ideaforge
WorkingDirectory=/opt/ideaforge/current
EnvironmentFile=/opt/ideaforge/shared/.env
ExecStart=/opt/ideaforge/current/ideaforge
Restart=on-failure
RestartSec=5s

# --- Resource caps: protect Discourse & other co-tenants ---
MemoryMax=512M
MemoryHigh=384M
CPUQuota=50%
TasksMax=256

# --- Hardening ---
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
PrivateDevices=true
ReadWritePaths=/opt/ideaforge/shared
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX
RestrictNamespaces=true
LockPersonality=true
MemoryDenyWriteExecute=false  # Rust binary is fine; set true if verified
SystemCallArchitectures=native

[Install]
WantedBy=multi-user.target
```

```bash
systemctl daemon-reload
systemctl enable ideaforge
# Don't start it yet — no release is deployed.
```

Tune `MemoryMax` / `CPUQuota` after observing real usage; 512M/50% is a conservative starting point that guarantees Discourse keeps its headroom.

---

## 5. nginx Reverse Proxy

Create `/etc/nginx/sites-available/ideaforge`:

```nginx
# Upstream — the Axum binary bound to localhost only
upstream ideaforge_api {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 80;
    listen [::]:80;
    server_name ideaforge.yourdomain.org;

    # Let certbot handle the redirect to HTTPS after step 8.
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }
    location / {
        return 301 https://$host$request_uri;
    }
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name ideaforge.yourdomain.org;

    # Filled in by certbot in step 8
    # ssl_certificate     /etc/letsencrypt/live/ideaforge.yourdomain.org/fullchain.pem;
    # ssl_certificate_key /etc/letsencrypt/live/ideaforge.yourdomain.org/privkey.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Frontend: served directly from the symlinked release dir.
    # Long-cache hashed assets; never cache index.html (it references the
    # current SRI hashes, so clients must always fetch a fresh copy).
    root /opt/ideaforge/current/dist;

    location = /index.html {
        add_header Cache-Control "no-store, must-revalidate" always;
        try_files /index.html =404;
    }

    # Trunk-hashed assets (e.g. ideaforge-abc123.wasm, style-def456.css)
    location ~* \.(wasm|js|css|png|jpg|jpeg|svg|woff2)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
        try_files $uri =404;
    }

    # API + WebSocket endpoints → Axum
    location /api/ {
        proxy_pass http://ideaforge_api;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 300s;
    }

    # SPA fallback: unknown paths → index.html (Leptos router handles the rest)
    location / {
        try_files $uri /index.html;
    }

    client_max_body_size 25m;
}
```

```bash
ln -s /etc/nginx/sites-available/ideaforge /etc/nginx/sites-enabled/
nginx -t && systemctl reload nginx
```

---

## 6. Server-Side Deploy Script

Place at `/opt/ideaforge/deploy.sh`, owned by `deploy:deploy`, mode `755`. GitHub Actions invokes this *after* scp-ing the release tarball to `/tmp/ideaforge-release.tar.gz`.

```bash
#!/bin/bash
# =============================================================================
# IdeaForge Deploy Script
# Location: /opt/ideaforge/deploy.sh
# Invoked by GitHub Actions after uploading /tmp/ideaforge-release.tar.gz
# =============================================================================
set -euo pipefail

APP_DIR="/opt/ideaforge"
RELEASES_DIR="$APP_DIR/releases"
CURRENT="$APP_DIR/current"
TARBALL="/tmp/ideaforge-release.tar.gz"
LOG_FILE="/var/log/ideaforge/deploy.log"
KEEP_RELEASES=5

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"; }

mkdir -p "$(dirname "$LOG_FILE")"
log "=== Deploy started ==="

if [[ ! -f "$TARBALL" ]]; then
    log "ERROR: $TARBALL not found"
    exit 1
fi

# Derive release name from the tarball's embedded metadata (set by CI)
RELEASE_NAME="$(date -u '+%Y-%m-%dT%H-%M-%S')-${GIT_SHA:-unknown}"
RELEASE_DIR="$RELEASES_DIR/$RELEASE_NAME"

log "Extracting to $RELEASE_DIR"
sudo -u ideaforge mkdir -p "$RELEASE_DIR"
sudo -u ideaforge tar -xzf "$TARBALL" -C "$RELEASE_DIR"
chmod +x "$RELEASE_DIR/ideaforge"

# Run migrations against the new binary BEFORE swapping traffic
log "Running migrations..."
sudo -u ideaforge env $(grep -v '^#' "$APP_DIR/shared/.env" | xargs) \
    "$RELEASE_DIR/migrate" up

# Atomic symlink swap
log "Swapping symlink $CURRENT -> $RELEASE_DIR"
sudo -u ideaforge ln -sfn "$RELEASE_DIR" "$CURRENT.new"
sudo -u ideaforge mv -Tf "$CURRENT.new" "$CURRENT"

# Restart the Axum binary (fast — sub-second)
log "Restarting ideaforge.service..."
sudo /bin/systemctl restart ideaforge

# Reload nginx so it picks up the new dist/ inode under the symlink.
# (Not strictly required — root path resolves lazily — but cheap insurance
# against any worker caching a stale fd.)
sudo /bin/systemctl reload nginx

# Health check
log "Health check..."
for i in {1..10}; do
    if curl -fsS http://127.0.0.1:8080/api/health > /dev/null; then
        log "Healthy"
        break
    fi
    [[ $i -eq 10 ]] && { log "ERROR: health check failed"; exit 1; }
    sleep 1
done

# Prune old releases
log "Pruning old releases (keeping $KEEP_RELEASES)..."
cd "$RELEASES_DIR"
ls -1t | tail -n +$((KEEP_RELEASES + 1)) | xargs -r -I{} rm -rf "{}"

rm -f "$TARBALL"
log "=== Deploy complete: $RELEASE_NAME ==="
```

**Rollback** (no rebuild needed):

```bash
cd /opt/ideaforge/releases
ls -1t                              # pick the previous release
sudo -u ideaforge ln -sfn releases/<PREV> /opt/ideaforge/current
sudo systemctl restart ideaforge
```

---

## 7. GitHub Actions Pipeline

### `.github/workflows/tests.yml`

```yaml
name: Tests
on:
  workflow_call:
  pull_request:
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: ideaforge
          POSTGRES_PASSWORD: ideaforge
          POSTGRES_DB: ideaforge_test
        ports: [5432:5432]
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    env:
      DATABASE_URL: postgres://ideaforge:ideaforge@localhost:5432/ideaforge_test
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: "1.85"
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: "src -> target"
      - name: cargo check
        run: cargo check --manifest-path src/Cargo.toml --workspace
      - name: cargo test
        run: cargo test --manifest-path src/Cargo.toml --workspace
```

### `.github/workflows/deploy.yml`

```yaml
name: Deploy
on:
  push:
    branches: [main]

concurrency:
  group: deploy-production
  cancel-in-progress: false

jobs:
  test:
    uses: ./.github/workflows/tests.yml

  build:
    name: Build release artifacts
    needs: test
    runs-on: ubuntu-latest
    outputs:
      sha: ${{ steps.meta.outputs.sha }}
    steps:
      - uses: actions/checkout@v4
      - id: meta
        run: echo "sha=$(git rev-parse --short HEAD)" >> $GITHUB_OUTPUT

      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: "1.85"
          targets: wasm32-unknown-unknown

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: "src -> target"

      - name: Install Trunk
        run: |
          curl -sSL https://github.com/trunk-rs/trunk/releases/download/v0.21.4/trunk-x86_64-unknown-linux-gnu.tar.gz \
            | tar -xz -C /usr/local/bin

      - name: Build API binary + migrate binary
        run: |
          cargo build --manifest-path src/Cargo.toml \
            --release --bin ideaforge --bin migrate

      - name: Build frontend (Trunk)
        working-directory: src/crates/ideaforge-frontend
        run: trunk build --release

      - name: Package release tarball
        run: |
          mkdir -p release
          cp src/target/release/ideaforge release/
          cp src/target/release/migrate release/
          cp -r src/crates/ideaforge-frontend/dist release/dist
          tar -czf ideaforge-release.tar.gz -C release .

      - uses: actions/upload-artifact@v4
        with:
          name: ideaforge-release
          path: ideaforge-release.tar.gz
          retention-days: 7

  deploy:
    name: Deploy to production
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          name: ideaforge-release

      - name: Upload tarball via scp
        uses: appleboy/scp-action@v0.1.7
        with:
          host: ${{ secrets.DEPLOY_HOST }}
          username: ${{ secrets.DEPLOY_USER }}
          key: ${{ secrets.DEPLOY_SSH_KEY }}
          source: "ideaforge-release.tar.gz"
          target: "/tmp/"

      - name: Run deploy script
        uses: appleboy/ssh-action@v1
        with:
          host: ${{ secrets.DEPLOY_HOST }}
          username: ${{ secrets.DEPLOY_USER }}
          key: ${{ secrets.DEPLOY_SSH_KEY }}
          envs: GIT_SHA
          script: |
            export GIT_SHA=${{ needs.build.outputs.sha }}
            /opt/ideaforge/deploy.sh
        env:
          GIT_SHA: ${{ needs.build.outputs.sha }}
```

Required GitHub secrets:

| Secret | Value |
|---|---|
| `DEPLOY_HOST` | Hetzner server IP |
| `DEPLOY_USER` | `deploy` |
| `DEPLOY_SSH_KEY` | Private half of the ed25519 key whose public half is in `/home/deploy/.ssh/authorized_keys` |

Generate the deploy key locally with `ssh-keygen -t ed25519 -C github-actions-ideaforge -f ideaforge_deploy -N ''`, paste the `.pub` into `authorized_keys` on the server, and paste the private half into GitHub → Settings → Secrets → Actions.

---

## 8. SSL via Let's Encrypt

Run only after DNS has propagated:

```bash
mkdir -p /var/www/certbot
certbot --nginx -d ideaforge.yourdomain.org \
    --non-interactive --agree-tos -m ops@yourdomain.org
```

Certbot will edit `/etc/nginx/sites-available/ideaforge` in place, uncommenting and filling in the `ssl_certificate` lines. Renewal is handled by the system-wide `certbot.timer` that Discourse/StoneWalker already rely on.

---

## 9. First Deploy

1. Push the `.github/workflows/` files and this doc to `main`.
2. Watch the Actions tab: Tests → Build → Deploy.
3. On the server:
   ```bash
   systemctl status ideaforge
   journalctl -u ideaforge -f
   ```
4. Visit `https://ideaforge.yourdomain.org` — SPA should load, `/api/health` should return 200.

---

## 10. Operations

### Monitoring co-tenant impact

```bash
# Watch memory usage across all three apps
systemctl status ideaforge discourse stonewalker --no-pager | grep -E 'Memory|CPU'

# If IdeaForge is getting OOM-killed, raise MemoryMax in the unit file.
# If Discourse slows down during deploys, lower CPUQuota to 30%.
```

### Viewing logs

```bash
journalctl -u ideaforge -f                    # live
journalctl -u ideaforge --since '1 hour ago'  # recent
tail -f /var/log/ideaforge/deploy.log         # deploy history
```

### Database backups

Add to root's crontab (adjust path to match existing backup conventions):

```cron
0 3 * * * pg_dump -U ideaforge ideaforge | gzip > /var/backups/ideaforge/$(date +\%Y\%m\%d).sql.gz && find /var/backups/ideaforge -mtime +14 -delete
```

### Emergency rollback

```bash
cd /opt/ideaforge/releases && ls -1t
sudo -u ideaforge ln -sfn /opt/ideaforge/releases/<PREV> /opt/ideaforge/current
sudo systemctl restart ideaforge
```

Run migrations backwards manually if the bad release included a forward migration.

---

## 11. Security Notes

- The `deploy` user has `NOPASSWD` sudo for *only* `systemctl restart ideaforge` and `systemctl reload nginx` — nothing else.
- The `ideaforge` user has no login shell.
- `/opt/ideaforge/shared/.env` is mode `600`, owned by `ideaforge`.
- Axum binds to `127.0.0.1` only — nginx is the sole public ingress.
- Never commit the real `.env`, Blockfrost keys, Stripe keys, or JWT secrets. Use `.env.template` in the repo as a placeholder.
- systemd hardening (`ProtectSystem=strict`, `PrivateTmp`, etc.) limits blast radius of any exploit.

---

## 12. Open Questions for the Dev Team

1. **Health endpoint**: does `/api/health` exist in `ideaforge-api`? If not, add a trivial `GET /api/health` that returns `200 OK` for the deploy script's health check.
2. **Migrations binary**: confirm `migrate` is built with `cargo build --bin migrate`. Adjust the deploy script's migrate step if the command-line interface differs.
3. **NATS & Blockfrost**: the `.env` template assumes both are reachable. For MVP launch, decide whether NATS runs on the same box (another systemd unit) or is external.
4. **Search index (Tantivy)**: needs a persistent writable path. Suggest `/opt/ideaforge/shared/search-index/` — add to `ReadWritePaths` in the systemd unit if so.
5. **Resource caps tuning**: after one week in production, review `systemd-cgtop` output and right-size `MemoryMax` / `CPUQuota`.
