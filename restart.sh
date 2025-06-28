#!/usr/bin/env sh

launchctl unload ~/Library/LaunchAgents/ckampfe.journal.plist
launchctl load -w ~/Library/LaunchAgents/ckampfe.journal.plist
