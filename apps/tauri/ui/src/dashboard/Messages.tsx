// The messages page: one card per message, whose body wraps to the width of the card.
import { For } from "solid-js";

import { spec } from "../spec";

/** The feed of message cards: the avatar, the author and the time on one line, the body below. */
export function Messages() {
  return (
    <div class="content__body feed">
      <For each={spec.messages}>
        {(message) => (
          <div class="card message">
            <span class="message__avatar" style={{ "background-color": message.avatar }}>
              {message.initials}
            </span>
            <div class="message__text">
              <div class="message__head">
                <span class="message__author">{message.author}</span>
                <span class="message__time">{message.time}</span>
              </div>
              <div class="message__body">{message.body}</div>
            </div>
          </div>
        )}
      </For>
    </div>
  );
}
