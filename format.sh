
for DIR in $PWD/backend $PWD/ui; do
    cd $DIR
    cargo fmt
done
