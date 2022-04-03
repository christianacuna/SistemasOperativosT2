#!/usr/bin/env python3
def main():
    filename = "test2.txt"
    output= "pepe2"
    f = open(filename, "r+")
    f.read()
    f.seek(0)
    f.write(output)
    f.truncate()
    f.close()
    return
if __name__ == "__main__":
    main()