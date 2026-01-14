#!/bin/bash

# A simple bash script to test indexing

GLOBAL_VAR="hello"

# function_definition
backup_logs() {
    local log_dir="/var/log/myapp"
    echo "Backing up logs from $log_dir"
    cp -r "$log_dir" /backup/
}

# if_statement (depth 1)
if [ -d "/var/log/myapp" ]; then
    backup_logs
else
    echo "Log directory not found"
fi

# expression_statement / command (depth 1)
ls -la /var/log/myapp
