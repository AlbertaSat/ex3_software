# EX3 Ground Station

## Overview

The EX3 Ground Station software interfaces with the Ex Alta 3 satellite, offering command and control capabilities through a web-based dashboard and a command-line interface. The project is organized as a Rust workspace with multiple components including a backend server, a web dashboar, and a CLI for direct OBC communication.

## Workspace Structure

- **cli_command_obc**: Command-line interface for direct communication with the satellite's on-board computer (OBC).
- **dashboard**: Web-based UI dashboard for monitoring and managing satellite operations.
- **server**: Backend server that provides APIs for the dashboard and handles all server-side logic including file-based data storage.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Trunk](https://trunkrs.dev/#install)

### Installation and Setup

**Clone the repository:**

- `git clone https://github.com/AlbertaSat/ex3_software.git`
- `cd ex3_ground_station`

### Run Server

Note: Data Storage
All data is stored inside the `server/data` folder as JSON files. The `data` directory and necessary JSON files will be created automatically when user sends requests to the server.

1. `cd server`
2. `cargo run`

### Run Dashboard

1.  `cd dashboard`
2.  `trunk serve`

### Run CLI

1. `cd cli_command_obc`
2. `cargo run`




