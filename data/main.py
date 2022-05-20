import pandas

if __name__=="__main__":
    chunksize = 1000000
    colors = set()
    with pandas.read_csv("sorted.csv.gz",
                            chunksize=chunksize,
                            compression='gzip',
                            names=["date", "uid", "color", "pos"]) as reader:
        for df in reader:
            _colors = df["color"].unique()
            colors.update(_colors)
            print(colors)
            print(df["date"].tail(1))