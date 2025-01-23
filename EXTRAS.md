# Inspiration

I like being on top of the news, but I don't like having to constantly check different feeds for important events. Google News scratches that itch, but it too is not up to my demands. Semi-inspired by S2's underground [The Wire](https://www.youtube.com/@S2Underground), I wanted a quick briefing on global events with only the most important information - however a lot of news sites has spam or a lot of local news that I don't care about. 

Instead, my idea is to check to identify the events that multiple sites are talking about, hypothetically because they must be more important and thus more "global". On top of calculating these clusters, I also rank them by how large they are, how recent, and how important they sound. Altogether, this forms the core of AutoInt.


# How it works

1. Gather news headlines from various rss feeds
2. Convert into word embeddings using [RustPotion](https://github.com/aveygo/rustpotion)
3. Identify clusters
4. Score clusters based on size, recency, and importance
5. Return found headlines

# Performance

Because I want to know immediately when events occur, performance is critical. This is why I use rust and my own custom libraries for embedding calculation and clustering, with multithreading and async functions.
I was able to achieve sentence embedding, clustering, and sorting in under 18ms. As a point of reference, it takes 32 ms to resolve the google news domain before the website is even requested.


# Rating

Numbers can be quite annoying to read, so we use a rating system. Below are the very rough targets that the rating is meant to represent.

### High

- AA: World altering event
- AB: Large military actions
- BB: Large political actions
- BC: Lives at immediate danger

### Medium

- CC: Weather warnings
- CD: Local crime

### Low

- DD: Minor Business news
- DE: Interesting story
- EE: Summaries or opinions
- EF: Sport
- FF: Celebrity news

<br>
Obviously, AutoInt is not going to exactly match these requirements but it should be good enough for 90% of cases.
<br>
<br>


# Updates

```0.1.0``` First release

```0.2.0``` Included html/icons within the binary itself & added some new sources 

```0.3.0``` Added rss feed timouts and can now handle arbitrary amounts of sources



# Future work

It should be possible to add a bunch of sources other than rss feeds like telegram or twitter, but that will wait for now. 
