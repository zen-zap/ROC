#!/bin/bash

echo "Starting the ROC Server"
(cd rocs && cargo run &)

sleep 2

echo "Starting the ROC Driver"
(cd rocd && cargo run)
