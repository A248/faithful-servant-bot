/*
 * faithful-servant-bot
 * Copyright © 2022 Anand Beh
 *
 * faithful-servant-bot is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * faithful-servant-bot is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with faithful-servant-bot. If not, see <https://www.gnu.org/licenses/>
 * and navigate to version 3 of the GNU General Public License.
 */

use std::borrow::Cow;
use std::collections::HashSet;
use once_cell::sync::OnceCell;

// All valid English words
// This is a rough measure. It isn't exact and doesn't include many American spellings
static ALL_WORDS_INDEX: OnceCell<HashSet<&'static str>> = OnceCell::new();

fn create_all_words_index() -> HashSet<&'static str> {
    // Words from http://www.mieliestronk.com/wordlist.html
    let all_words = include_str!("corncob_lowercase.txt");
    let mut all_words = all_words.split_terminator('\n').collect::<HashSet<_>>();
    all_words.shrink_to_fit();
    all_words
}

pub fn count_words<C: AsRef<str>>(content: C) -> u32 {
    let all_words_index = ALL_WORDS_INDEX.get_or_init(create_all_words_index);

    let mut word_count = 0;
    let content = content.as_ref();
    for word in content.split(' ') {
        let word = word.to_lowercase();
        if all_words_index.contains(word.as_str()) {
            word_count += 1;
        }
    }
    word_count
}

pub fn respond_to_message<C: AsRef<str>>(_content: C) -> Option<Cow<'static, str>> {
    // Todo: Add responses to direct address, inquiries, etc.
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_legal_words() {
        assert_eq!(8, count_words("This is a legal word but this is not : ohuhasiudnakj"))
    }
}
