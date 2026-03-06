# Privacy-Preserving Auctions

## Description

This project implements a **privacy-preserving sealed-bid auction system**.  
Users can register, submit encrypted bids, and participate in an auction without revealing losing bids.  
The winner is determined in a transparent and verifiable way, and each losing bidder can publish a **Proof Certificate** demonstrating that their bid was lower than the winning price without revealing the actual value.  

A public **Bulletin Board** stores all bids, proofs, and related data to enable offline verification of the auction results.