# Octavia
Octavia is a Minecraft server written in Rust.

As of writing this README, it's very very barebone (you can log-in and that's it, no entity update).

## Motivation

The goal of this project was to learn how the Minecraft Protocol works. Granted I've learned a ton !

I'm putting it up for public because I don't actively work on it anymore. Feel free to take a peek at how I implemented parts of the protocol!

I was going to write a test suite at some point to show how little work I've achieved, but I never finished that and lost the local changes from that test suite. The gist of it was to use a tool that would log the flow of a working client and compare it with mine. You'd end up with a delta diff to indicate how much is missing.
