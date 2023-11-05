use std::{hint, time::Instant};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use smallvec::SmallVec;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Serialize, Deserialize)]
#[repr(align(64))]
struct Post<'a> {
    #[serde(rename = "_id")]
    id: &'a str,
    title: &'a str,
    tags: Vec<&'a str>,
}

const NUM_TOP_ITEMS: usize = 5;

#[derive(Serialize)]
struct RelatedPosts<'a, const TOPN: usize> {
    #[serde(rename = "_id")]
    id: &'a str,
    tags: &'a [&'a str],
    related: SmallVec<[&'a Post<'a>; TOPN]>,
}

fn main() {
    let json_str = std::fs::read_to_string("../posts.json").unwrap();
    let posts: Vec<Post> = from_str(&json_str).unwrap();
    assert!(posts.len() > 5, "current implementation panics if less than 5 posts.");

    let start = hint::black_box(Instant::now());

    let mut post_tags_map: FxHashMap<&str, SmallVec<[u32; 32]>> = FxHashMap::default();

    for (post_idx, post) in posts.iter().enumerate() {
        for tag in &post.tags {
            post_tags_map.entry(tag).or_default().push(post_idx as u32);
        }
    }

    let mut related_posts: Vec<RelatedPosts<'_, NUM_TOP_ITEMS>> = Vec::with_capacity(posts.len());
    let mut tagged_post_count: Vec<u8> = Vec::with_capacity(posts.len());
    tagged_post_count.resize(posts.len(), 0);

    for (post_idx, post) in posts.iter().enumerate() {
        if post_idx > 0 { // avoid first unnecessary memset
            tagged_post_count.fill(0);
        }

        for tag in &post.tags {
            if let Some(tag_posts) = post_tags_map.get(tag) {
                for other_post_idx in tag_posts {
                    tagged_post_count[*other_post_idx as usize] += 1;
                }
            }
        }
        tagged_post_count[post_idx] = 0; // don't recommend the same post

        let mut top_n_counts: [u8; NUM_TOP_ITEMS] = [0u8; NUM_TOP_ITEMS];
        let mut top_n_posts: [&Post; NUM_TOP_ITEMS] = [&posts[0], &posts[1], &posts[2], &posts[3], &posts[4]];
        let mut min_tags = 0u8;
        for (post, count) in tagged_post_count.iter().copied().enumerate() {
            if count > min_tags {
                let mut i = NUM_TOP_ITEMS - 1;
                while i > 0 && top_n_counts[i - 1] < count {
                    // rotate top_n
                    top_n_counts[i] = top_n_counts[i - 1];
                    top_n_posts[i] = top_n_posts[i - 1];
                    i -= 1;
                }
                // insert into top_n
                top_n_counts[i] = count;
                top_n_posts[i] = &posts[post];

                min_tags = top_n_counts[NUM_TOP_ITEMS - 1];
            }
        }

        related_posts.push(RelatedPosts {
            id: post.id,
            tags: &post.tags,
            related: SmallVec::from_slice(&top_n_posts),
        });
    }

    // Tell compiler to not delay now() until print is eval'ed.
    let end = hint::black_box(Instant::now());

    println!("Processing time (w/o IO): {:?}", end.duration_since(start));

    // I have no explanation for why, but doing this before the print improves performance pretty
    // significantly (15%) when using slices in the hashmap key and RelatedPosts
    let json_str = serde_json::to_string(&related_posts).unwrap();

    std::fs::write("../related_posts_rust.json", json_str).unwrap();
}
