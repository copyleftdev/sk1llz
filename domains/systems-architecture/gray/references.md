# Jim Gray: References

## Essential Papers

### Transaction Processing

- **"The Transaction Concept: Virtues and Limitations"** (1981)
  - Foundational paper defining transactions
  - [Microsoft Research](https://www.microsoft.com/en-us/research/publication/the-transaction-concept-virtues-and-limitations/)

- **"Granularity of Locks and Degrees of Consistency"** (1976)
  - With Lorie, Putzolu, Traiger
  - Lock hierarchies and isolation levels
  - [Microsoft Research](https://www.microsoft.com/en-us/research/publication/granularity-locks-degrees-consistency-multi-level-sharing/)

### Recovery

- **"ARIES: A Transaction Recovery Method"** (1992)
  - Mohan, Haderle, Lindsay, Pirahesh, Schwarz
  - The gold standard for database recovery
  - [IBM Research](https://dl.acm.org/doi/10.1145/128765.128770)

### The Five-Minute Rule

- **"The Five-Minute Rule for Trading Memory for Disk Accesses"** (1987)
  - Gray & Putzolu
  - [ACM SIGMOD Record](https://dl.acm.org/doi/10.1145/38714.38755)

- **"The Five-Minute Rule Ten Years Later"** (1997)
  - Gray & Graefe
  - Updated analysis with SSDs in view

### Benchmarking

- **"A Measure of Transaction Processing Power"** (1985)
  - The DebitCredit benchmark, precursor to TPC-A
  - [Datamation](https://dl.acm.org/doi/10.1145/318898.318912)

### Fault Tolerance

- **"Why Do Computers Stop and What Can Be Done About It?"** (1985)
  - Landmark study of failure causes
  - [Tandem Technical Report](https://www.hpl.hp.com/techreports/tandem/TR-85.7.pdf)

## Books

### By Jim Gray

- **"Transaction Processing: Concepts and Techniques"** (1992)
  - Gray & Reuter
  - THE definitive textbook on transaction processing
  - ISBN: 978-1558601901

### About Gray's Work

- **"Readings in Database Systems"** (Red Book)
  - Hellerstein & Stonebraker (editors)
  - Multiple Gray papers included
  - [Online edition](http://www.redbook.io/)

## Talks and Lectures

- **Turing Award Lecture** (1998)
  - "What Next? A Dozen Remaining IT Problems"
  - [ACM Digital Library](https://dl.acm.org/doi/10.1145/276305.276314)

- **SIGMOD Keynotes**
  - Multiple influential talks on database evolution

## TPC Benchmarks

Gray founded the Transaction Processing Performance Council:

- **TPC-C**: Online transaction processing
- **TPC-H**: Decision support (analytics)
- **TPC-E**: Current OLTP benchmark

All specifications at: [TPC.org](http://www.tpc.org/)

## Awards and Recognition

- **ACM Turing Award** (1998) — "for seminal contributions to database and transaction processing research"
- **IEEE John von Neumann Medal** (2005)
- **ACM SIGMOD Edgar F. Codd Award** (1993)

## Historical Context

### Tandem Computers (1970s–1990s)
- Gray worked on NonStop systems
- Pioneered fault-tolerant transaction processing

### Microsoft Research (1995–2007)
- Led the Bay Area Research Center
- Worked on scalable computing, databases, and eScience

## Related Researchers

- **Andreas Reuter**: Co-author of Transaction Processing book
- **C. Mohan**: ARIES recovery algorithm
- **Phil Bernstein**: Concurrency control
- **Mike Stonebraker**: Database systems (complementary work)

## Memorials

Jim Gray disappeared at sea on January 28, 2007. His legacy includes:

- **Jim Gray eScience Award** (given by Microsoft Research)
- **Jim Gray Seed Grant** (UC Berkeley)
- Multiple conference sessions dedicated to his memory

## Courses Using These Materials

- **CMU 15-445: Database Systems** (Andy Pavlo)
  - Extensive coverage of recovery and transactions
  - [Course page](https://15445.courses.cs.cmu.edu/)

- **Stanford CS245: Principles of Data-Intensive Systems**
  - Transaction processing foundations
