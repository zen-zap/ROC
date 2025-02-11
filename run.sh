#!/bin/bash

echo "Starting the ROC Server"
gnome-terminal -- bash -c "cd rocs && cargo run; exec bash"

sleep 2

echo "Starting the ROC Driver"
gnome-terminal -- bash -c "cd rocd && cargo run; exec bash"
