#!/bin/bash
set -e

# Interactive Build Script for Linux

function show_menu() {
    clear
    echo -e "\033[0;36m===========================\033[0m"
    echo -e "\033[0;36m   Code-RAG Build Menu\033[0m"
    echo -e "\033[0;36m===========================\033[0m"
    echo "1. Build (Debug)"
    echo "2. Build (Release)"
    echo "3. Build (Release - CUDA)"
    echo "Q. Quit"
    echo -e "\033[0;36m===========================\033[0m"
}

while true; do
    show_menu
    read -p "Select an option: " choice
    case $choice in
        1)
            echo -e "\033[0;33mStarting Debug Build...\033[0m"
            cargo build --bin code-rag
            echo -e "\033[0;32mDone. Binary: target/debug/code-rag\033[0m"
            read -p "Press Enter to return to menu..."
            ;;
        2)
            echo -e "\033[0;33mStarting Release Build...\033[0m"
            cargo build --release --bin code-rag
            echo -e "\033[0;32mDone. Binary: target/release/code-rag\033[0m"
            read -p "Press Enter to return to menu..."
            ;;
        3)
            echo -e "\033[0;33mStarting CUDA Release Build...\033[0m"
            cargo build --release --features cuda --bin code-rag
            cp target/release/code-rag target/release/code-rag-cuda
            echo -e "\033[0;32mDone. Binary: target/release/code-rag-cuda\033[0m"
            read -p "Press Enter to return to menu..."
            ;;
        [Qq]*)
            echo -e "\033[0;32mExiting...\033[0m"
            exit 0
            ;;
        *)
            echo -e "\033[0;31mInvalid selection. Please try again.\033[0m"
            sleep 1
            ;;
    esac
done
