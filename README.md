# props2yaml

- Before printing the output, the YML is formatted with [ymlfmt](https://github.com/google/yamlfmt).
This can be skipped with `--skip-format`
- If `yamlfmt` is not in the PATH, it is also possible to specify the path to the executable with `--yamlfmt-path`

## Docker

```shell
$ docker build -t props2yaml .
$ docker run --rm -v $(pwd):props2yml/ props2yaml <args...>
```
