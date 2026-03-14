#!/usr/bin/env bash
set -euo pipefail

# Seed mock data for Saga blog post screenshots
# Creates 5 clients, 10 projects, 6 tags, 3 rates, 34 time entries, and 1 active timer

echo "=== Seeding Saga with mock data ==="

# ─── Configuration ───────────────────────────────────────────────────────────
echo "Setting configuration..."
saga config set default_currency "USD"
saga config set daily_goal_hours "8"
saga config set weekly_goal_hours "40"
saga config set default_hourly_rate "125"
saga config set default_billable "true"

# ─── Clients ─────────────────────────────────────────────────────────────────
echo "Creating clients..."
saga clients add "Meridian Labs"    --contact "Sarah Chen"      --email "sarah@meridianlabs.io"
saga clients add "Foxglove Agency"  --contact "Jamie Ortiz"     --email "jamie@foxglove.design"
saga clients add "Stratos Cloud"    --contact "Raj Patel"       --email "raj@stratoscloud.dev"
saga clients add "Pinecone Ventures" --contact "Lena Kowalski"  --email "lena@pineconevc.com"
saga clients add "Wildframe Studio" --contact "Marcus Ng"       --email "marcus@wildframe.co"

# ─── Projects ────────────────────────────────────────────────────────────────
echo "Creating projects..."
saga projects add "Meridian API"         -c "Meridian Labs"     --color "#5B9BD5" -b 120
saga projects add "Meridian Dashboard"   -c "Meridian Labs"     --color "#3498DB" -b 80
saga projects add "Foxglove Rebrand"     -c "Foxglove Agency"   --color "#ED7D31" -b 60
saga projects add "Foxglove CMS"         -c "Foxglove Agency"   --color "#E74C3C"
saga projects add "Stratos Migration"    -c "Stratos Cloud"     --color "#A569BD" -b 200
saga projects add "Stratos Monitoring"   -c "Stratos Cloud"     --color "#1ABC9C" -b 40
saga projects add "Pinecone Pitch Deck"  -c "Pinecone Ventures" --color "#F1C40F" -b 15
saga projects add "Wildframe Portfolio"  -c "Wildframe Studio"  --color "#70AD47" -b 30
saga projects add "Internal - Saga Dev"                         --color "#9B59B6"
saga projects add "Old Rebrand"          -c "Foxglove Agency"   --color "#808080"
saga projects archive "Old Rebrand"

# ─── Tags ────────────────────────────────────────────────────────────────────
echo "Creating tags..."
saga tags add "backend"  --color "#3498DB"
saga tags add "frontend" --color "#E74C3C"
saga tags add "devops"   --color "#27AE60"
saga tags add "design"   --color "#F39C12"
saga tags add "meeting"  --color "#9B59B6"
saga tags add "planning" --color "#95A5A6"

# ─── Rates ───────────────────────────────────────────────────────────────────
echo "Setting rates..."
saga rates set 125                                    # Default rate
saga rates set 175 --client "Stratos Cloud"           # Premium client rate
saga rates set 200 --project "Pinecone Pitch Deck"    # High-touch project rate

# ─── Time Entries ────────────────────────────────────────────────────────────
echo "Adding time entries..."

# ── Monday 2026-03-09 (7h 30m — 6 entries) ──
saga add -p "Meridian API"        -s "2026-03-09 09:00" -e "2026-03-09 10:00" -d "Sprint planning kickoff"              -t meeting -t planning
saga add -p "Meridian API"        -s "2026-03-09 10:15" -e "2026-03-09 12:00" -d "REST endpoint scaffolding"             -t backend
saga add -p "Foxglove Rebrand"    -s "2026-03-09 13:00" -e "2026-03-09 14:30" -d "Design token system setup"             -t design -t frontend
saga add -p "Foxglove Rebrand"    -s "2026-03-09 14:45" -e "2026-03-09 15:30" -d "Color palette exploration"             -t design
saga add -p "Stratos Migration"   -s "2026-03-09 15:45" -e "2026-03-09 17:15" -d "Terraform VPC provisioning"            -t devops
saga add -p "Meridian API"        -s "2026-03-09 17:15" -e "2026-03-09 18:15" -d "Database schema review"                -t backend -t planning

# ── Tuesday 2026-03-10 (8h 15m — 6 entries) ──
saga add -p "Foxglove Rebrand"    -s "2026-03-10 09:00" -e "2026-03-10 10:15" -d "Brand guidelines review with client"   -t meeting -t design
saga add -p "Meridian API"        -s "2026-03-10 10:30" -e "2026-03-10 12:15" -d "Rate limiting middleware"              -t backend
saga add -p "Stratos Migration"   -s "2026-03-10 13:00" -e "2026-03-10 15:00" -d "RDS cluster configuration"             -t devops
saga add -p "Wildframe Portfolio" -s "2026-03-10 15:15" -e "2026-03-10 16:45" -d "Gallery component build"               -t frontend
saga add -p "Meridian API"        -s "2026-03-10 16:45" -e "2026-03-10 17:45" -d "API error handling patterns"           -t backend
saga add -p "Internal - Saga Dev" -s "2026-03-10 17:45" -e "2026-03-10 18:30" -d "README and docs cleanup"               -t planning

# ── Wednesday 2026-03-11 (8h 45m — 6 entries) ──
saga add -p "Stratos Migration"   -s "2026-03-11 09:00" -e "2026-03-11 10:30" -d "Load testing harness setup"            -t devops -t backend
saga add -p "Meridian Dashboard"  -s "2026-03-11 10:45" -e "2026-03-11 13:00" -d "OAuth2 provider integration"           -t backend
saga add -p "Wildframe Portfolio" -s "2026-03-11 13:45" -e "2026-03-11 15:30" -d "Hero section animations"               -t frontend -t design
saga add -p "Pinecone Pitch Deck" -s "2026-03-11 15:45" -e "2026-03-11 17:15" -d "Pitch deck wireframes"                 -t design
saga add -p "Meridian API"        -s "2026-03-11 17:15" -e "2026-03-11 18:00" -d "Sprint retrospective"                  -t meeting
saga add -p "Meridian Dashboard"  -s "2026-03-11 18:00" -e "2026-03-11 19:00" -d "Dashboard chart components"            -t frontend

# ── Thursday 2026-03-12 (9h 00m — 6 entries) ──
saga add -p "Meridian API"        -s "2026-03-12 09:00" -e "2026-03-12 10:30" -d "Code review - PR batch"                -t backend
saga add -p "Stratos Migration"   -s "2026-03-12 10:45" -e "2026-03-12 13:00" -d "Legacy data migration scripts"         -t backend
saga add -p "Foxglove CMS"        -s "2026-03-12 13:45" -e "2026-03-12 15:15" -d "CMS content model design"              -t backend -t planning
saga add -p "Foxglove CMS"        -s "2026-03-12 15:15" -e "2026-03-12 16:15" -d "CMS API endpoints"                     -t backend
saga add -p "Meridian Dashboard"  -s "2026-03-12 16:30" -e "2026-03-12 18:30" -d "WebSocket event handlers"              -t backend -t frontend
saga add -p "Stratos Monitoring"  -s "2026-03-12 18:30" -e "2026-03-12 19:15" -d "Client status update call"             -t meeting

# ── Friday 2026-03-13 (7h 45m — 5 entries) ──
saga add -p "Meridian API"        -s "2026-03-13 09:00" -e "2026-03-13 11:15" -d "Integration test suite"                -t backend
saga add -p "Stratos Migration"   -s "2026-03-13 11:30" -e "2026-03-13 13:00" -d "Staging deploy sign-off"               -t devops
saga add -p "Foxglove Rebrand"    -s "2026-03-13 13:45" -e "2026-03-13 15:30" -d "Typography and color system"           -t design -t frontend
saga add -p "Meridian Dashboard"  -s "2026-03-13 15:45" -e "2026-03-13 17:30" -d "Performance audit tooling"             -t frontend -t backend
saga add -p "Internal - Saga Dev" -s "2026-03-13 17:30" -e "2026-03-13 18:00" -d "Invoice prep and time review"          -t planning

# ── Saturday 2026-03-14 (3h 15m completed — 4 entries) ──
saga add -p "Meridian API"        -s "2026-03-14 08:30" -e "2026-03-14 09:30" -d "N+1 query optimization"                -t backend
saga add -p "Stratos Monitoring"  -s "2026-03-14 09:45" -e "2026-03-14 10:45" -d "Prometheus alerting rules"             -t devops
saga add -p "Internal - Saga Dev" -s "2026-03-14 11:00" -e "2026-03-14 12:00" -d "Open issue triage"                     -t planning
saga add -p "Pinecone Pitch Deck" -s "2026-03-14 12:15" -e "2026-03-14 12:30" -d "Pitch deck final polish"               -t design

# ─── Active Timer ────────────────────────────────────────────────────────────
echo "Starting active timer..."
saga start "Foxglove Rebrand" -d "Responsive testimonials carousel" -t frontend

# ─── Flip a few entries to non-billable for realistic reports ────────────────
echo "Marking some entries as non-billable..."
sqlite3 ~/.local/share/saga/saga.db \
  "UPDATE time_entries SET billable=0 WHERE description LIKE '%planning%' OR description LIKE '%README%' OR description LIKE '%triage%';"

echo ""
echo "=== Seeding complete! ==="
echo ""

# ─── Verification ────────────────────────────────────────────────────────────
echo "--- Verification ---"
echo ""
echo "Clients:"
saga clients list
echo ""
echo "Projects:"
saga projects list --all
echo ""
echo "Tags:"
saga tags list
echo ""
echo "Rates:"
saga rates list
echo ""
echo "Status:"
saga status
echo ""
echo "This week's log:"
saga log --week
