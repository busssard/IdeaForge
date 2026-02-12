#!/usr/bin/env bash
# IdeaForge API Integration Tests
# Run with: bash tests/test_api.sh
set -euo pipefail

BASE="http://localhost:3000"
PASS=0
FAIL=0
TOTAL=0

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

assert_status() {
    local test_name="$1"
    local expected="$2"
    local actual="$3"
    TOTAL=$((TOTAL + 1))
    if [ "$actual" -eq "$expected" ]; then
        echo -e "  ${GREEN}PASS${NC} $test_name (HTTP $actual)"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}FAIL${NC} $test_name (expected $expected, got $actual)"
        FAIL=$((FAIL + 1))
    fi
}

assert_json_field() {
    local test_name="$1"
    local body="$2"
    local field="$3"
    local expected="$4"
    TOTAL=$((TOTAL + 1))
    local actual
    actual=$(echo "$body" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d${field})" 2>/dev/null || echo "__PARSE_ERROR__")
    if [ "$actual" = "$expected" ]; then
        echo -e "  ${GREEN}PASS${NC} $test_name ($field = $expected)"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}FAIL${NC} $test_name ($field: expected '$expected', got '$actual')"
        FAIL=$((FAIL + 1))
    fi
}

assert_json_exists() {
    local test_name="$1"
    local body="$2"
    local field="$3"
    TOTAL=$((TOTAL + 1))
    local actual
    actual=$(echo "$body" | python3 -c "import sys,json; d=json.load(sys.stdin); v=d${field}; print('exists' if v is not None else 'none')" 2>/dev/null || echo "__PARSE_ERROR__")
    if [ "$actual" = "exists" ]; then
        echo -e "  ${GREEN}PASS${NC} $test_name ($field exists)"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}FAIL${NC} $test_name ($field missing or null)"
        FAIL=$((FAIL + 1))
    fi
}

assert_header() {
    local test_name="$1"
    local headers="$2"
    local header_name="$3"
    local expected_value="$4"
    TOTAL=$((TOTAL + 1))
    if echo "$headers" | grep -qi "${header_name}:.*${expected_value}"; then
        echo -e "  ${GREEN}PASS${NC} $test_name ($header_name present)"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}FAIL${NC} $test_name ($header_name: expected '$expected_value')"
        FAIL=$((FAIL + 1))
    fi
}

# Helper: HTTP request returning status code + body
# Usage: read status body <<< $(http GET /path)
http() {
    local method="$1"
    local path="$2"
    local auth="${3:-}"
    local data="${4:-}"

    local args=(-s -w "\n%{http_code}" -X "$method" "${BASE}${path}")
    args+=(-H "Content-Type: application/json")
    if [ -n "$auth" ]; then
        args+=(-H "Authorization: Bearer $auth")
    fi
    if [ -n "$data" ]; then
        args+=(--data-binary "$data")
    fi

    local response
    response=$(curl "${args[@]}" 2>/dev/null)
    local status_code
    status_code=$(echo "$response" | tail -1)
    local body
    body=$(echo "$response" | sed '$d')

    echo "$status_code"
    echo "$body"
}

echo ""
echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}  IdeaForge API Integration Tests${NC}"
echo -e "${YELLOW}========================================${NC}"
echo ""

# ─────────────────────────────────────────
echo -e "${YELLOW}[1] Health Check${NC}"
# ─────────────────────────────────────────
RESP=$(http GET /health)
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "GET /health" 200 "$STATUS"
assert_json_field "Health status" "$BODY" "['status']" "ok"
assert_json_field "Health DB" "$BODY" "['database']" "connected"

# Check security headers
HEADERS=$(curl -sI "${BASE}/health" 2>/dev/null)
assert_header "X-Content-Type-Options" "$HEADERS" "x-content-type-options" "nosniff"
assert_header "X-Frame-Options" "$HEADERS" "x-frame-options" "DENY"
assert_header "X-XSS-Protection" "$HEADERS" "x-xss-protection" "1; mode=block"
assert_header "Referrer-Policy" "$HEADERS" "referrer-policy" "strict-origin-when-cross-origin"
assert_header "Content-Security-Policy" "$HEADERS" "content-security-policy" "default-src 'self'"

# ─────────────────────────────────────────
echo ""
echo -e "${YELLOW}[2] Auth: Registration${NC}"
# ─────────────────────────────────────────
TIMESTAMP=$(date +%s)
REG_EMAIL="testuser${TIMESTAMP}@example.com"
REG_JSON=$(cat <<ENDJSON
{"email":"${REG_EMAIL}","password":"SecurePass1","display_name":"Integration Tester"}
ENDJSON
)

RESP=$(http POST /api/v1/auth/register "" "$REG_JSON")
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "POST /auth/register" 201 "$STATUS"
assert_json_exists "Has access_token" "$BODY" "['access_token']"
assert_json_exists "Has refresh_token" "$BODY" "['refresh_token']"
assert_json_field "Token type" "$BODY" "['token_type']" "Bearer"

USER_TOKEN=$(echo "$BODY" | python3 -c "import sys,json; print(json.load(sys.stdin)['access_token'])" 2>/dev/null)
USER_ID=$(echo "$BODY" | python3 -c "import sys,json; print(json.load(sys.stdin)['user_id'])" 2>/dev/null)
REFRESH_TOKEN=$(echo "$BODY" | python3 -c "import sys,json; print(json.load(sys.stdin)['refresh_token'])" 2>/dev/null)

# Duplicate registration
RESP=$(http POST /api/v1/auth/register "" "$REG_JSON")
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "Duplicate registration" 409 "$STATUS"
assert_json_field "Conflict error code" "$BODY" "['error']['code']" "CONFLICT"

# Validation: weak password
RESP=$(http POST /api/v1/auth/register "" '{"email":"weak@example.com","password":"weak","display_name":"Weak"}')
STATUS=$(echo "$RESP" | head -1)
assert_status "Weak password rejected" 400 "$STATUS"

# Validation: bad email
RESP=$(http POST /api/v1/auth/register "" '{"email":"notanemail","password":"SecurePass1","display_name":"Bad"}')
STATUS=$(echo "$RESP" | head -1)
assert_status "Invalid email rejected" 400 "$STATUS"

# ─────────────────────────────────────────
echo ""
echo -e "${YELLOW}[3] Auth: Login${NC}"
# ─────────────────────────────────────────
LOGIN_JSON=$(cat <<ENDJSON
{"email":"${REG_EMAIL}","password":"SecurePass1"}
ENDJSON
)

RESP=$(http POST /api/v1/auth/login "" "$LOGIN_JSON")
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "POST /auth/login" 200 "$STATUS"
assert_json_exists "Login returns access_token" "$BODY" "['access_token']"

# Wrong password
RESP=$(http POST /api/v1/auth/login "" "{\"email\":\"${REG_EMAIL}\",\"password\":\"WrongPass1\"}")
STATUS=$(echo "$RESP" | head -1)
assert_status "Wrong password" 401 "$STATUS"

# Non-existent user
RESP=$(http POST /api/v1/auth/login "" '{"email":"noone@example.com","password":"SecurePass1"}')
STATUS=$(echo "$RESP" | head -1)
assert_status "Non-existent user" 401 "$STATUS"

# ─────────────────────────────────────────
echo ""
echo -e "${YELLOW}[4] Auth: Token Refresh${NC}"
# ─────────────────────────────────────────
RESP=$(http POST /api/v1/auth/refresh "" "{\"refresh_token\":\"${REFRESH_TOKEN}\"}")
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "POST /auth/refresh" 200 "$STATUS"
assert_json_exists "Refresh returns new access_token" "$BODY" "['access_token']"

# Invalid refresh token
RESP=$(http POST /api/v1/auth/refresh "" '{"refresh_token":"invalid.token.here"}')
STATUS=$(echo "$RESP" | head -1)
assert_status "Invalid refresh token" 401 "$STATUS"

# ─────────────────────────────────────────
echo ""
echo -e "${YELLOW}[5] User Profile${NC}"
# ─────────────────────────────────────────
RESP=$(http GET /api/v1/users/me "$USER_TOKEN")
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "GET /users/me" 200 "$STATUS"
assert_json_field "Profile email" "$BODY" "['email']" "$REG_EMAIL"
assert_json_field "Profile display_name" "$BODY" "['display_name']" "Integration Tester"
assert_json_field "Profile role" "$BODY" "['role']" "curious"

# Update profile
RESP=$(http PUT /api/v1/users/me "$USER_TOKEN" '{"display_name":"Updated Name","bio":"Hello world"}')
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "PUT /users/me" 200 "$STATUS"
assert_json_field "Updated display_name" "$BODY" "['display_name']" "Updated Name"
assert_json_field "Updated bio" "$BODY" "['bio']" "Hello world"

# Public profile
RESP=$(http GET "/api/v1/users/${USER_ID}")
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "GET /users/:id (public)" 200 "$STATUS"
assert_json_field "Public profile name" "$BODY" "['display_name']" "Updated Name"

# Unauthenticated /me
RESP=$(http GET /api/v1/users/me)
STATUS=$(echo "$RESP" | head -1)
assert_status "GET /users/me unauthenticated" 401 "$STATUS"

# ─────────────────────────────────────────
echo ""
echo -e "${YELLOW}[6] Categories${NC}"
# ─────────────────────────────────────────
RESP=$(http GET /api/v1/categories)
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "GET /categories" 200 "$STATUS"

CAT_COUNT=$(echo "$BODY" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null)
TOTAL=$((TOTAL + 1))
if [ "$CAT_COUNT" -eq 8 ]; then
    echo -e "  ${GREEN}PASS${NC} 8 categories seeded"
    PASS=$((PASS + 1))
else
    echo -e "  ${RED}FAIL${NC} Expected 8 categories, got $CAT_COUNT"
    FAIL=$((FAIL + 1))
fi

RESP=$(http GET /api/v1/categories/technology)
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "GET /categories/:slug" 200 "$STATUS"
assert_json_field "Category name" "$BODY" "['name']" "Technology"

RESP=$(http GET /api/v1/categories/nonexistent)
STATUS=$(echo "$RESP" | head -1)
assert_status "GET /categories/:slug not found" 404 "$STATUS"

# ─────────────────────────────────────────
echo ""
echo -e "${YELLOW}[7] Ideas CRUD${NC}"
# ─────────────────────────────────────────

# Create idea (authenticated)
IDEA_JSON='{"title":"Integration Test Idea","summary":"A test idea created by the integration tests","description":"Detailed description for integration testing purposes.","openness":"collaborative"}'
RESP=$(http POST /api/v1/ideas "$USER_TOKEN" "$IDEA_JSON")
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "POST /ideas (create)" 201 "$STATUS"
assert_json_field "Idea title" "$BODY" "['title']" "Integration Test Idea"
assert_json_field "Idea maturity" "$BODY" "['maturity']" "spark"
assert_json_field "Idea openness" "$BODY" "['openness']" "collaborative"
assert_json_field "Idea stoke_count" "$BODY" "['stoke_count']" "0"

IDEA_ID=$(echo "$BODY" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null)

# Create unauthenticated (should fail)
RESP=$(http POST /api/v1/ideas "" "$IDEA_JSON")
STATUS=$(echo "$RESP" | head -1)
assert_status "POST /ideas unauthenticated" 401 "$STATUS"

# Validation: empty title
RESP=$(http POST /api/v1/ideas "$USER_TOKEN" '{"title":"","summary":"ok","description":"ok"}')
STATUS=$(echo "$RESP" | head -1)
assert_status "Empty title rejected" 400 "$STATUS"

# Get single idea (public)
RESP=$(http GET "/api/v1/ideas/${IDEA_ID}")
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "GET /ideas/:id" 200 "$STATUS"
assert_json_field "Get idea title" "$BODY" "['title']" "Integration Test Idea"

# List ideas (public)
RESP=$(http GET /api/v1/ideas)
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "GET /ideas (list)" 200 "$STATUS"
assert_json_exists "Has data array" "$BODY" "['data']"
assert_json_exists "Has pagination meta" "$BODY" "['meta']"

LIST_COUNT=$(echo "$BODY" | python3 -c "import sys,json; print(json.load(sys.stdin)['meta']['total'])" 2>/dev/null)
TOTAL=$((TOTAL + 1))
if [ "$LIST_COUNT" -ge 4 ]; then
    echo -e "  ${GREEN}PASS${NC} Ideas list has $LIST_COUNT items (3 seeded + 1 created)"
    PASS=$((PASS + 1))
else
    echo -e "  ${RED}FAIL${NC} Expected >= 4 ideas, got $LIST_COUNT"
    FAIL=$((FAIL + 1))
fi

# Update idea (owner)
RESP=$(http PUT "/api/v1/ideas/${IDEA_ID}" "$USER_TOKEN" '{"title":"Updated Integration Idea","summary":"Updated summary"}')
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "PUT /ideas/:id (owner)" 200 "$STATUS"
assert_json_field "Updated title" "$BODY" "['title']" "Updated Integration Idea"
assert_json_field "Updated summary" "$BODY" "['summary']" "Updated summary"

# Update idea by non-owner (login as seeded Alice, try to update our idea)
ALICE_LOGIN=$(cat <<'ENDJSON'
{"email":"alice@example.com","password":"Test1234!"}
ENDJSON
)
RESP=$(http POST /api/v1/auth/login "" "$ALICE_LOGIN")
ALICE_BODY=$(echo "$RESP" | tail -n +2)
ALICE_TOKEN=$(echo "$ALICE_BODY" | python3 -c "import sys,json; print(json.load(sys.stdin)['access_token'])" 2>/dev/null)

RESP=$(http PUT "/api/v1/ideas/${IDEA_ID}" "$ALICE_TOKEN" '{"title":"Hacked Title"}')
STATUS=$(echo "$RESP" | head -1)
assert_status "PUT /ideas/:id (non-owner)" 403 "$STATUS"

# Get non-existent idea
RESP=$(http GET "/api/v1/ideas/00000000-0000-0000-0000-000000000000")
STATUS=$(echo "$RESP" | head -1)
assert_status "GET /ideas/:id not found" 404 "$STATUS"

# ─────────────────────────────────────────
echo ""
echo -e "${YELLOW}[8] Stokes (Idea Endorsements)${NC}"
# ─────────────────────────────────────────

# Stoke an idea
RESP=$(http POST "/api/v1/ideas/${IDEA_ID}/stokes" "$USER_TOKEN")
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "POST /ideas/:id/stokes" 201 "$STATUS"
assert_json_field "Stoke idea_id" "$BODY" "['idea_id']" "$IDEA_ID"

# Double stoke (conflict)
RESP=$(http POST "/api/v1/ideas/${IDEA_ID}/stokes" "$USER_TOKEN")
STATUS=$(echo "$RESP" | head -1)
assert_status "Double stoke rejected" 409 "$STATUS"

# Verify stoke count updated
RESP=$(http GET "/api/v1/ideas/${IDEA_ID}")
BODY=$(echo "$RESP" | tail -n +2)
assert_json_field "Stoke count incremented" "$BODY" "['stoke_count']" "1"

# Alice also stokes
RESP=$(http POST "/api/v1/ideas/${IDEA_ID}/stokes" "$ALICE_TOKEN")
STATUS=$(echo "$RESP" | head -1)
assert_status "Alice stokes idea" 201 "$STATUS"

# Verify stoke count is now 2
RESP=$(http GET "/api/v1/ideas/${IDEA_ID}")
BODY=$(echo "$RESP" | tail -n +2)
assert_json_field "Stoke count is 2" "$BODY" "['stoke_count']" "2"

# List stokes
RESP=$(http GET "/api/v1/ideas/${IDEA_ID}/stokes")
STATUS=$(echo "$RESP" | head -1)
BODY=$(echo "$RESP" | tail -n +2)
assert_status "GET /ideas/:id/stokes" 200 "$STATUS"

STOKE_COUNT=$(echo "$BODY" | python3 -c "import sys,json; print(json.load(sys.stdin)['meta']['total'])" 2>/dev/null)
TOTAL=$((TOTAL + 1))
if [ "$STOKE_COUNT" -eq 2 ]; then
    echo -e "  ${GREEN}PASS${NC} Stokes list shows 2 stokes"
    PASS=$((PASS + 1))
else
    echo -e "  ${RED}FAIL${NC} Expected 2 stokes, got $STOKE_COUNT"
    FAIL=$((FAIL + 1))
fi

# Withdraw stoke
RESP=$(http DELETE "/api/v1/ideas/${IDEA_ID}/stokes/mine" "$USER_TOKEN")
STATUS=$(echo "$RESP" | head -1)
assert_status "DELETE /ideas/:id/stokes/mine" 204 "$STATUS"

# Verify stoke count back to 1
RESP=$(http GET "/api/v1/ideas/${IDEA_ID}")
BODY=$(echo "$RESP" | tail -n +2)
assert_json_field "Stoke count after withdrawal" "$BODY" "['stoke_count']" "1"

# Stoke unauthenticated
RESP=$(http POST "/api/v1/ideas/${IDEA_ID}/stokes")
STATUS=$(echo "$RESP" | head -1)
assert_status "Stoke unauthenticated" 401 "$STATUS"

# ─────────────────────────────────────────
echo ""
echo -e "${YELLOW}[9] Idea Archive (Soft Delete)${NC}"
# ─────────────────────────────────────────

# Archive by non-owner should fail
RESP=$(http DELETE "/api/v1/ideas/${IDEA_ID}" "$ALICE_TOKEN")
STATUS=$(echo "$RESP" | head -1)
assert_status "DELETE /ideas/:id (non-owner)" 403 "$STATUS"

# Archive by owner
RESP=$(http DELETE "/api/v1/ideas/${IDEA_ID}" "$USER_TOKEN")
STATUS=$(echo "$RESP" | head -1)
assert_status "DELETE /ideas/:id (owner)" 204 "$STATUS"

# Archived idea should not appear
RESP=$(http GET "/api/v1/ideas/${IDEA_ID}")
STATUS=$(echo "$RESP" | head -1)
assert_status "Archived idea is gone" 404 "$STATUS"

# ─────────────────────────────────────────
echo ""
echo -e "${YELLOW}[10] Seeded Data Verification${NC}"
# ─────────────────────────────────────────

# Seeded ideas should exist
RESP=$(http GET "/api/v1/ideas?per_page=100")
BODY=$(echo "$RESP" | tail -n +2)

HAS_AI_TUTOR=$(echo "$BODY" | python3 -c "
import sys,json
d=json.load(sys.stdin)
print('yes' if any(i['title']=='Open Source AI Tutor' for i in d['data']) else 'no')
" 2>/dev/null)
TOTAL=$((TOTAL + 1))
if [ "$HAS_AI_TUTOR" = "yes" ]; then
    echo -e "  ${GREEN}PASS${NC} Seeded idea 'Open Source AI Tutor' exists"
    PASS=$((PASS + 1))
else
    echo -e "  ${RED}FAIL${NC} Seeded idea 'Open Source AI Tutor' not found"
    FAIL=$((FAIL + 1))
fi

# Seeded user Alice can login (already tested above, but verify role)
RESP=$(http GET /api/v1/users/me "$ALICE_TOKEN")
BODY=$(echo "$RESP" | tail -n +2)
assert_json_field "Alice role" "$BODY" "['role']" "entrepreneur"

# ─────────────────────────────────────────
# Summary
# ─────────────────────────────────────────
echo ""
echo -e "${YELLOW}========================================${NC}"
if [ "$FAIL" -eq 0 ]; then
    echo -e "  ${GREEN}ALL $TOTAL TESTS PASSED${NC}"
else
    echo -e "  ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC} out of $TOTAL"
fi
echo -e "${YELLOW}========================================${NC}"
echo ""

exit "$FAIL"
