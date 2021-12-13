#!/bin/bash
heroku config:set DISCORD_TOKEN=
heroku buildpacks:set emk/rust
heroku buildpacks:add https://github.com/appositum/heroku-buildpack-youtube-dl.git
heroku buildpacks:add https://github.com/jonathanong/heroku-buildpack-ffmpeg-latest.git
heroku buildpacks:add https://github.com/MyJules/heroku-buildpack-libopus.git
