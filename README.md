# Deep Atlantic Storage

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/yoursunny/summer-host-storage/build.yml?style=flat)](https://github.com/yoursunny/summer-host-storage/actions) [![GitHub code size](https://img.shields.io/github/languages/code-size/yoursunny/summer-host-storage?style=flat&logo=GitHub)](https://github.com/yoursunny/summer-host-storage/)

![Deep Atlantic Storage logo](docs/logo.svg)

**Deep Atlantic Storage** is a fictional hosting service for entertainment purposes.
The service offers unlimited free storage for finite sized files of any size.
It is equipped with advanced sorting technology that keeps your data neatly ordered.
Every uploaded file is sorted by its bits, and a URI is returned that allows you to download the same file with the same bits.

## Installation

You must have a Rust toolchain including `cargo`.
Enter this command to download and install this amusing application:

```bash
cargo install --git https://github.com/yoursunny/summer-host-storage.git
```

**Deep Atlantic Storage app** is then available as `yoursunny_summer_host_storage` command.
This command allows you to use Deep Atlantic Storage service without an Internet connection.

## CLI Usage

```bash
# upload a file, URI is printed to stdout
yoursunny_summer_host_storage upload pushups.mp4

# upload a file from stdin, URI is printed to stdout
yoursunny_summer_host_storage upload --stdin pushups.mp4 <pushups.mp4

# download a file
yoursunny_summer_host_storage download https://summer-host-storage.yoursunny.dev/100002230/ffffddd0/pushups.mp4

# download a file to stdout
yoursunny_summer_host_storage download --stdout https://summer-host-storage.yoursunny.dev/100002230/ffffddd0/pushups.mp4 >pushups.mp4
```

## HTTP Usage

```bash
# start the HTTP server
yoursunny_summer_host_storage serve --bind [::1]:3000

# upload a file, URI is returned in Location header
http POST http://[::1]:3000/upload/pushups.mp4 @pushups.mp4

# download a file
http GET http://[::1]:3000/100002230/ffffddd0/pushups.mp4
```
