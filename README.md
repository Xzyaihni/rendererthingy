# rendererthingy
cons:
1) unoptimized code
2) runs on the cpu
3) single threaded

pros:

# how to install or something
```
git clone https://github.com/Xzyaihni/rendererthingy
cd rendererthingy
cargo b -r
./target/release/rendererthingy -d 5 defaultmodels/cube.obj
```

will create an image named output.ppm

```
./target/release/rendererthingy -m console -d 5 defaultmodels/cube.obj
```

will do pretty ascii art

ok bye
