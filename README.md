# Local Search Engine in Rust

**WORK IN PROGRESS**

## References

- [Search Engine in Rust (Ep.01) by Tsoding Daily](https://www.youtube.com/watch?v=hm5xOJiVEeg)
- [Rust Web Development (Search Engine Ep.02) by Tsoding Daily](https://www.youtube.com/watch?v=OYAKjlYm_Ew&t=5957s)

## Abbreviations and Shorthands

| Shorthand   | Meaning                             | Description                                                                                                                                        |
| ----------- | ----------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tf`        | Term Frequency                      |                                                                                                                                                    |
| `idf`       | Inverse Document Frequency          |                                                                                                                                                    |
| `dtf`       | Document Terms Frequencies          | The HashMap that stores the frequencies of terms that occur in the document, with the term itself as the key and the frequency count as the value. |
| `dtf_index` | Index of Document Terms Frequencies | The HashMap that stores Document Terms Frequencies with the file path as key.                                                                      |
