#!/bin/bash
# sd can be installed via:
#
#   cargo install sd

# Get an UUIDv4.
# Response looks like e.g.:
#
#   ["0a280cfc-50ad-11ed-90b4-fffe4ec3e6dc"]
uuid=$(curl -s https://www.uuidtools.com/api/generate/v1)

# sd to remove [ ] "
echo $uuid | sd '\[|\]|"' ''
