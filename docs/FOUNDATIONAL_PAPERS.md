# QR Neural Memory System — Foundational Research

Curated 2026-11-12. Research reading list, not a demonstration that QR Neural Memory works. Links lead to primary publications, authors' copies, or archival records. PDFs are not republished here.

## Foundational associative and distributed memory

### John J. Hopfield (1982)
**Neural networks and physical systems with emergent collective computational abilities.** *PNAS* 79(8), 2554–2558.
- Original article: https://doi.org/10.1073/pnas.79.8.2554
- Open full text: https://pmc.ncbi.nlm.nih.gov/articles/PMC346238/
- Relevance: collective, content-addressable recall; reconstructing patterns from incomplete states. Original Hopfield update assumptions should not be conflated with our proposed synchronous local rules.

### Pentti Kanerva (1988)
**Sparse Distributed Memory.** MIT Press. This is a **book**, not a journal paper.
- Publisher: https://mitpress.mit.edu/9780262514699/sparse-distributed-memory/
- NASA bibliographic record: https://ntrs.nasa.gov/citations/19900032242
- Relevance: high-dimensional similarity-addressed retrieval and distributed representations. Unrestricted full-book PDF not established.

### Pentti Kanerva (1992)
**Sparse Distributed Memory and Related Models.** RIACS Technical Report 92.10 / NASA-CR-190553.
- NASA record: https://ntrs.nasa.gov/citations/19920021480
- NASA PDF: https://ntrs.nasa.gov/api/citations/19920021480/downloads/19920021480.pdf?attachment=true
- Relevance: an accessible technical overview to read with the monograph.

## Hiroshi Inazawa — priority research

### Hiroshi Inazawa and Kazuhisa Kitakaze (1998)
**A locally inhibited lateral neural network: on the fundamental features of a network consisting of neurons with a restricted range of interaction.** *Biological Cybernetics* 78(3), 207–215.
- DOI: https://doi.org/10.1007/s004220050427
- Abstract: https://pubmed.ncbi.nlm.nih.gov/9602524/
- Relevance: restricted local interactions, lateral inhibition, and pattern dynamics. Full open PDF not established.

### Hiroshi Inazawa (2018)
**Hetero-Correlation-Associative Memory with Trigger Neurons: Accumulation of Memory through Additional Learning in Neural Networks.** *Complex Systems* 27(2), 187–197.
- DOI: https://doi.org/10.25088/ComplexSystems.27.2.187
- Publisher-hosted PDF: https://content.wolfram.com/sites/13/2018/09/27-2-6.pdf
- Relevance: sequential acquisition without erasing earlier associations. Compare mechanisms carefully: adding trigger neurons is not the same as learning in our fixed-size substrate.

### Hiroshi Inazawa (2022)
**An associative memory model with very high memory rate: Image storage by sequential addition learning.** arXiv:2210.03893.
- Paper: https://arxiv.org/abs/2210.03893
- PDF: https://arxiv.org/pdf/2210.03893
- Relevance: sequential storage, Cue Ball / Recall Net and retention claims. Check paper definitions before any quantitative comparisons.

### Hiroshi Inazawa (2025)
**The Method for Storing Patterns in Neural Networks-Memorization and Recall of QR code Patterns-.** arXiv:2504.06631.
- Paper: https://arxiv.org/abs/2504.06631
- PDF: https://arxiv.org/pdf/2504.06631
- Relevance: partial QR-code-pattern cues and reconstruction. **Crucial difference:** QR codes are data patterns in this paper; our hypothetical QR neurons contain mutable internal structure.

### Hiroshi Inazawa and Garrison W. Cottrell (2006)
**Phase space learning in an autonomous dynamical neural network.** *Neurocomputing* 69(16–18), 2340–2345.
- DOI: https://doi.org/10.1016/j.neucom.2006.04.006
- Author-hosted PDF: https://cseweb.ucsd.edu/~gary/pubs/inazawa-neurocomputing-2006.pdf
- Relevance: dynamic and online learning.

### Hiroshi Inazawa (2025)
**Associative Memory Model with Neural Networks: Memorizing multiple images with one neuron.** arXiv:2510.06542.
- Paper: https://arxiv.org/abs/2510.06542
- PDF: https://arxiv.org/pdf/2510.06542
- Relevance: contrasting neuron-specific storage with the proposed distributed overlapping trace.

### Hiroshi Inazawa (2025)
**Associative Memory using Attribute-Specific Neuron Groups-1: Learning between Multiple Cue Balls.** arXiv:2512.02319.
- Paper: https://arxiv.org/abs/2512.02319
- PDF: https://arxiv.org/pdf/2512.02319
- Relevance: coupled association between represented attributes.

### Hiroshi Inazawa (2026)
**Associative Memory using Attribute-Specific Neuron Groups-2: Learning and Sequential Associative Recall between Cue Neurons for different Cue Balls.** arXiv:2603.24910.
- Paper: https://arxiv.org/abs/2603.24910
- PDF: https://arxiv.org/pdf/2603.24910
- Relevance: chained recall and association sequences.

### Hiroshi Inazawa (2026)
**Memory and Recall of Color Images Using Neural Networks-1 - The Cue-Ball Model based on the Associative Memory Mechanism -.** arXiv:2610.41108.
- Paper: https://arxiv.org/abs/2610.41108
- PDF: https://arxiv.org/pdf/2610.41108
- Note: Verify bibliographic metadata from the live arXiv page before treating this as a confirmed reference.

## Proposed experimental comparisons

| Proposed property | Relevant prior work | Falsification-oriented test |
|---|---|---|
| Associative recovery from incomplete cues | Hopfield, Kanerva, Inazawa 2025 QR | Trained vs untrained, correct vs unrelated partial cues |
| Retention after successive associations | Inazawa 2018 and 2022 | Train A→B, then C→D; re-test A→B |
| Locality of interactions | Inazawa and Kitakaze 1998 | Eight-neighbor vs matched-budget wider-radius interactions |
| Damage resilience | Hopfield; Kanerva | Random failures, clustered lesions, disconnected regions |
| Contribution of QR-internal structure | Inazawa 2025 QR (different use) | Structured neuron vs equally resourced simple-cell baseline |

## Status and cautions

1. A citation does not constitute reproduction or validation.
2. Distributed storage is not automatically fault tolerant; overlapping representations and retrieval dynamics must be measured.
3. The current GitHub scaffold contains 4,096 binary **cells**, not verified implementations of 4,096 internally structured QR neurons.
4. Whether the learning trace belongs to per-neuron state, edges, or sustained activation must be specified.
5. Keep published full-text PDFs outside a public repository until redistribution rights are confirmed.
6. Research correspondence is private and not included in public documentation.
