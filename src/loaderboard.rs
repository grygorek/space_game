// Copyright 2026 Piotr Grygorczuk <grygorek@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use std::fs;

pub struct Leaderboard {
    pub entries: Vec<(String, u32)>,
    file_path: &'static str,
}

impl Leaderboard {
    pub fn new() -> Self {
        let mut lb = Self { entries: Vec::new(), file_path: "scores.txt" };
        lb.load();
        lb
    }

    pub fn load(&mut self) {
        if let Ok(content) = fs::read_to_string(self.file_path) {
            self.entries = content
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() == 2 {
                        let name = parts[0].to_string();
                        let score = parts[1].parse().ok()?;
                        Some((name, score))
                    } else {
                        None
                    }
                })
                .collect();
        }

        // Ensure we always have 5 entries
        while self.entries.len() < 5 {
            self.entries.push(("CPU".to_string(), 10));
        }
        self.sort();
    }

    pub fn is_high_score(&self, score: u32) -> bool {
        self.entries.len() < 5 || score > self.entries.last().map(|e| e.1).unwrap_or(0)
    }

    pub fn add_entry(&mut self, name: String, score: u32) {
        self.entries.push((name.to_uppercase(), score));
        self.sort();
        self.entries.truncate(5);
        self.save();
    }

    fn sort(&mut self) {
        self.entries.sort_by(|a, b| b.1.cmp(&a.1));
    }

    fn save(&self) {
        let content = self.entries.iter().map(|(n, s)| format!("{}:{}", n, s)).collect::<Vec<_>>().join("\n");
        let _ = fs::write(self.file_path, content);
    }
}
