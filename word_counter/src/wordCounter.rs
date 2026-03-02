fn most_frequent_word(text: &str) -> (String, usize) {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return (String::new(), 0);
    }

    let mut visited = vec![false; words.len()];
    let mut max_idx = 0usize;
    let mut max_count = 0usize;

    for i in 0..words.len() {
        if visited[i] { continue; }
        visited[i] = true;

        let mut count = 1;
        for j in (i + 1)..words.len() {
            if !visited[j] && words[j] == words[i] {
                visited[j] = true;
                count += 1;
            }
        }
        if count > max_count {
            max_count = count;
            max_idx = i;
        }
    }
    (words[max_idx].to_string(), max_count)
}

fn main() {
    let text = "the quick brown fox jumps over the lazy dog the quick brown fox";
    let (word, count) = most_frequent_word(text);
    println!("Most frequent word: \"{}\" ({} times)", word, count);
}