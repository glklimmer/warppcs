#!/bin/bash

SESSION_NAME="warppcs"

# Check if tmux is installed
if ! command -v tmux &> /dev/null; then
    echo "tmux is not installed. Please install tmux and try again."
    exit 1
fi

# Function to create a new session
run_commands() {
    tmux send-keys -t 0 'cargo run --bin server' C-m
    tmux send-keys -t 1 'cargo run --bin client' C-m
    tmux send-keys -t 2 'cargo run --bin client' C-m
}

# Check if the session already exists
if tmux has-session -t $SESSION_NAME 2>/dev/null; then
    echo "Session $SESSION_NAME already exists. Attaching..."
    run_commands
    tmux attach-session -t $SESSION_NAME
else
    echo "Creating new session $SESSION_NAME..."
    tmux new-session -d -s $SESSION_NAME
    tmux split-window -h
    tmux split-window -v
    run_commands
    tmux attach-session -t $SESSION_NAME
fi
