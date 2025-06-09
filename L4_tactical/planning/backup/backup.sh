#!/bin/bash

# HAL9 Backup Script
# Comprehensive backup for the hierarchical architecture

set -e

# Configuration
BACKUP_TYPE=${1:-"incremental"}  # full, incremental, or state-only
NAMESPACE="hal9"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR="/backup/hal9/$TIMESTAMP"
S3_BUCKET="s3://hal9-backups/production"
RETENTION_DAYS=30

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}HAL9 Backup Utility${NC}"
echo -e "${BLUE}==================${NC}"
echo "Backup Type: $BACKUP_TYPE"
echo "Timestamp: $TIMESTAMP"
echo

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Function to backup PostgreSQL
backup_postgresql() {
    echo -e "${YELLOW}Backing up PostgreSQL databases...${NC}"
    
    local pg_pod=$(kubectl get pod -n $NAMESPACE -l app=postgresql,role=primary -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -z "$pg_pod" ]; then
        echo -e "${RED}PostgreSQL primary pod not found${NC}"
        return 1
    fi
    
    # Full backup
    if [ "$BACKUP_TYPE" == "full" ]; then
        echo "  Creating full database backup..."
        kubectl exec -n $NAMESPACE $pg_pod -- pg_dumpall -U postgres | \
            gzip > "$BACKUP_DIR/postgresql-full.sql.gz"
        
        echo -e "  ${GREEN}✓${NC} Full database backup completed"
    fi
    
    # Always backup critical tables
    local tables=("neuron_states" "learning_data" "signal_history" "consensus_records" "emergence_patterns")
    
    for table in "${tables[@]}"; do
        echo "  Backing up table: $table"
        kubectl exec -n $NAMESPACE $pg_pod -- pg_dump -U hal9 -d hal9 -t $table | \
            gzip > "$BACKUP_DIR/${table}.sql.gz"
    done
    
    echo -e "${GREEN}PostgreSQL backup completed${NC}"
}

# Function to backup Redis
backup_redis() {
    echo -e "${YELLOW}Backing up Redis data...${NC}"
    
    local redis_pod=$(kubectl get pod -n $NAMESPACE -l app=redis,role=master -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -z "$redis_pod" ]; then
        echo -e "${RED}Redis master pod not found${NC}"
        return 1
    fi
    
    # Trigger BGSAVE
    echo "  Triggering Redis background save..."
    kubectl exec -n $NAMESPACE $redis_pod -- redis-cli BGSAVE
    
    # Wait for save to complete
    echo "  Waiting for save to complete..."
    while [ "$(kubectl exec -n $NAMESPACE $redis_pod -- redis-cli LASTSAVE)" == "$(kubectl exec -n $NAMESPACE $redis_pod -- redis-cli LASTSAVE)" ]; do
        sleep 1
    done
    
    # Copy RDB file
    echo "  Copying Redis dump file..."
    kubectl cp $NAMESPACE/$redis_pod:/data/dump.rdb "$BACKUP_DIR/redis-dump.rdb"
    
    echo -e "${GREEN}Redis backup completed${NC}"
}

# Function to backup Kubernetes resources
backup_kubernetes() {
    echo -e "${YELLOW}Backing up Kubernetes resources...${NC}"
    
    # Backup all HAL9 resources
    echo "  Exporting Kubernetes manifests..."
    
    # Deployments
    kubectl get deployments -n $NAMESPACE -o yaml > "$BACKUP_DIR/deployments.yaml"
    
    # Services
    kubectl get services -n $NAMESPACE -o yaml > "$BACKUP_DIR/services.yaml"
    
    # ConfigMaps
    kubectl get configmaps -n $NAMESPACE -o yaml > "$BACKUP_DIR/configmaps.yaml"
    
    # Secrets (encrypted)
    kubectl get secrets -n $NAMESPACE -o yaml | \
        kubectl neat | \
        kubeseal --format yaml > "$BACKUP_DIR/sealed-secrets.yaml" 2>/dev/null || \
        kubectl get secrets -n $NAMESPACE -o yaml > "$BACKUP_DIR/secrets.yaml"
    
    # PVCs
    kubectl get pvc -n $NAMESPACE -o yaml > "$BACKUP_DIR/pvcs.yaml"
    
    # NetworkPolicies
    kubectl get networkpolicies -n $NAMESPACE -o yaml > "$BACKUP_DIR/networkpolicies.yaml"
    
    echo -e "${GREEN}Kubernetes resources backup completed${NC}"
}

# Function to backup neuron states
backup_neuron_states() {
    echo -e "${YELLOW}Backing up neuron states...${NC}"
    
    # Export neuron states from each layer
    LAYERS=("l1-reflexive" "l2-implementation" "l3-operational" "l4-tactical" "l5-strategic")
    
    for layer in "${LAYERS[@]}"; do
        echo "  Backing up $layer neuron states..."
        
        local pod=$(kubectl get pod -n $NAMESPACE -l layer=$layer -o jsonpath='{.items[0].metadata.name}' 2>/dev/null | head -1)
        
        if [ -n "$pod" ]; then
            kubectl exec -n $NAMESPACE $pod -- hal9-cli export neurons --format json | \
                gzip > "$BACKUP_DIR/neurons-${layer}.json.gz"
        fi
    done
    
    echo -e "${GREEN}Neuron states backup completed${NC}"
}

# Function to backup learning data
backup_learning_data() {
    echo -e "${YELLOW}Backing up learning data...${NC}"
    
    # Export learning patterns
    local cognitive_pod=$(kubectl get pod -n $NAMESPACE -l component=cognitive -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -n "$cognitive_pod" ]; then
        echo "  Exporting learning patterns..."
        kubectl exec -n $NAMESPACE $cognitive_pod -- hal9-cli export learning --all | \
            gzip > "$BACKUP_DIR/learning-patterns.json.gz"
        
        echo "  Exporting gradients..."
        kubectl exec -n $NAMESPACE $cognitive_pod -- hal9-cli export gradients --recent 1000 | \
            gzip > "$BACKUP_DIR/gradients.json.gz"
    fi
    
    echo -e "${GREEN}Learning data backup completed${NC}"
}

# Function to backup configuration
backup_configuration() {
    echo -e "${YELLOW}Backing up configuration...${NC}"
    
    # Export all configuration
    local orchestrator_pod=$(kubectl get pod -n $NAMESPACE -l component=orchestrator -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)
    
    if [ -n "$orchestrator_pod" ]; then
        kubectl exec -n $NAMESPACE $orchestrator_pod -- hal9-cli config export --all > \
            "$BACKUP_DIR/hal9-config.yaml"
    fi
    
    # Backup environment-specific configs
    cp -r /etc/hal9/* "$BACKUP_DIR/" 2>/dev/null || true
    
    echo -e "${GREEN}Configuration backup completed${NC}"
}

# Function to create backup manifest
create_manifest() {
    echo -e "${YELLOW}Creating backup manifest...${NC}"
    
    cat > "$BACKUP_DIR/manifest.json" <<EOF
{
    "timestamp": "$TIMESTAMP",
    "backup_type": "$BACKUP_TYPE",
    "hal9_version": "$(kubectl get deployment -n $NAMESPACE hal9-orchestrator -o jsonpath='{.spec.template.spec.containers[0].image}' | cut -d: -f2)",
    "components": {
        "postgresql": $([ -f "$BACKUP_DIR/postgresql-full.sql.gz" ] && echo "true" || echo "false"),
        "redis": $([ -f "$BACKUP_DIR/redis-dump.rdb" ] && echo "true" || echo "false"),
        "kubernetes": $([ -f "$BACKUP_DIR/deployments.yaml" ] && echo "true" || echo "false"),
        "neuron_states": $(ls "$BACKUP_DIR"/neurons-*.json.gz 2>/dev/null | wc -l),
        "learning_data": $([ -f "$BACKUP_DIR/learning-patterns.json.gz" ] && echo "true" || echo "false"),
        "configuration": $([ -f "$BACKUP_DIR/hal9-config.yaml" ] && echo "true" || echo "false")
    },
    "size_bytes": $(du -sb "$BACKUP_DIR" | cut -f1),
    "checksum": ""
}
EOF
    
    # Calculate checksum
    local checksum=$(find "$BACKUP_DIR" -type f ! -name "manifest.json" -exec sha256sum {} \; | sort | sha256sum | cut -d' ' -f1)
    sed -i "s/\"checksum\": \"\"/\"checksum\": \"$checksum\"/" "$BACKUP_DIR/manifest.json"
    
    echo -e "${GREEN}Manifest created${NC}"
}

# Function to upload to S3
upload_to_s3() {
    echo -e "${YELLOW}Uploading backup to S3...${NC}"
    
    # Compress entire backup
    echo "  Compressing backup..."
    tar -czf "/tmp/hal9-backup-$TIMESTAMP.tar.gz" -C "$(dirname $BACKUP_DIR)" "$(basename $BACKUP_DIR)"
    
    # Upload to S3
    echo "  Uploading to S3..."
    aws s3 cp "/tmp/hal9-backup-$TIMESTAMP.tar.gz" "$S3_BUCKET/$TIMESTAMP/hal9-backup-$TIMESTAMP.tar.gz" \
        --storage-class STANDARD_IA
    
    # Upload manifest separately for easy access
    aws s3 cp "$BACKUP_DIR/manifest.json" "$S3_BUCKET/$TIMESTAMP/manifest.json"
    
    # Clean up local compressed file
    rm -f "/tmp/hal9-backup-$TIMESTAMP.tar.gz"
    
    echo -e "${GREEN}Upload completed${NC}"
}

# Function to cleanup old backups
cleanup_old_backups() {
    echo -e "${YELLOW}Cleaning up old backups...${NC}"
    
    # Local cleanup
    find /backup/hal9 -type d -mtime +$RETENTION_DAYS -exec rm -rf {} \; 2>/dev/null || true
    
    # S3 cleanup (using lifecycle policy is preferred)
    # aws s3 ls $S3_BUCKET/ | awk '{print $2}' | while read dir; do
    #     # Parse date from directory name and check if older than retention
    # done
    
    echo -e "${GREEN}Cleanup completed${NC}"
}

# Function to verify backup
verify_backup() {
    echo -e "${YELLOW}Verifying backup integrity...${NC}"
    
    local errors=0
    
    # Check file sizes
    while IFS= read -r file; do
        if [ ! -s "$file" ]; then
            echo -e "  ${RED}✗ Empty file: $file${NC}"
            ((errors++))
        fi
    done < <(find "$BACKUP_DIR" -type f)
    
    # Verify PostgreSQL dumps
    for dump in "$BACKUP_DIR"/*.sql.gz; do
        if [ -f "$dump" ]; then
            if ! gzip -t "$dump" 2>/dev/null; then
                echo -e "  ${RED}✗ Corrupted SQL dump: $dump${NC}"
                ((errors++))
            fi
        fi
    done
    
    # Verify JSON files
    for json in "$BACKUP_DIR"/*.json; do
        if [ -f "$json" ]; then
            if ! jq empty "$json" 2>/dev/null; then
                echo -e "  ${RED}✗ Invalid JSON: $json${NC}"
                ((errors++))
            fi
        fi
    done
    
    if [ $errors -eq 0 ]; then
        echo -e "  ${GREEN}✓ All backup files verified${NC}"
        return 0
    else
        echo -e "  ${RED}✗ Found $errors errors in backup${NC}"
        return 1
    fi
}

# Main backup flow
echo "Starting HAL9 backup..."

# Run appropriate backup based on type
case $BACKUP_TYPE in
    "full")
        backup_postgresql
        backup_redis
        backup_kubernetes
        backup_neuron_states
        backup_learning_data
        backup_configuration
        ;;
    "incremental")
        backup_postgresql  # Critical tables only
        backup_redis
        backup_neuron_states
        backup_learning_data
        ;;
    "state-only")
        backup_neuron_states
        backup_learning_data
        ;;
    *)
        echo -e "${RED}Invalid backup type: $BACKUP_TYPE${NC}"
        echo "Valid types: full, incremental, state-only"
        exit 1
        ;;
esac

# Create manifest
create_manifest

# Verify backup
if verify_backup; then
    echo -e "${GREEN}Backup verification passed${NC}"
else
    echo -e "${RED}Backup verification failed${NC}"
    exit 1
fi

# Upload to S3
if [ -n "$AWS_ACCESS_KEY_ID" ]; then
    upload_to_s3
else
    echo -e "${YELLOW}Skipping S3 upload (AWS credentials not configured)${NC}"
fi

# Cleanup old backups
cleanup_old_backups

# Summary
echo
echo -e "${GREEN}Backup completed successfully!${NC}"
echo "Backup location: $BACKUP_DIR"
echo "Backup size: $(du -sh "$BACKUP_DIR" | cut -f1)"
echo "Manifest: $BACKUP_DIR/manifest.json"

if [ -n "$AWS_ACCESS_KEY_ID" ]; then
    echo "S3 location: $S3_BUCKET/$TIMESTAMP/"
fi

exit 0