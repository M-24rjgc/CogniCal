#!/usr/bin/env sh
if [ -z "$husky_skip_init" ]; then
  export husky_skip_init=1

  if [ -f /usr/share/husky/husky.sh ]; then
    . /usr/share/husky/husky.sh
  else
    if [ -f "$HOME/.husky/husky.sh" ]; then
      . "$HOME/.husky/husky.sh"
    fi
  fi
fi
