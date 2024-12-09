<p align=center>
    <img src="static/logo.svg" width=200px>
</p>

<h1 align=center id="user-content-toc">Automated Intelligence</h1>


</br>
</br>
</br>

# Inspiration

I like being on top of the news, but I don't like having to constantly check different feeds for important events. Google News scratches that itch, but it too is not up to my demands. Semi-inspired by S2's underground [The Wire](https://www.youtube.com/@S2Underground), I wanted a quick briefing on global events with only the most important information - however a lot of news sites has spam or a lot of local news that I don't care about. 

Instead, my idea is to check to identify the events that multiple sites are talking about, hypothetically because they must be more important and global. On top of calculating these clusters, I also rank them by how large they are, how recent, and how important they sound. Altogether, this forms the core of AutoInt.


# How it works

1. Gather news headlines from various rss feeds
2. Convert into word embeddings using [RustPotion](https://github.com/aveygo/rustpotion)
3. Identify clusters
4. Score clusters based on size, recency, and importance
5. Return found headlines

# Performance

Because I want to know immediately when events occur, performance is critical. This is why I use rust and my own custom libraries for embedding calculation and clustering, with multithreading and async functions.
I was able to achieve sentence embedding, clustering, and sorting in under 18ms. As a point of reference, it takes 32 ms to resolve the google news domain before the website is even requested.

# Self Hosting

TODO

# Cool facts

- Written in 🔥rust🔥
- Might break if you look at it wrong
- Running on a pi 🥧