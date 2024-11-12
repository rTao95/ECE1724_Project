# ECE1724_Project

## Link
* task tracker: https://docs.google.com/spreadsheets/d/1j89gKSbjLw0jRq4DW_lp3bVWRzagvjYnBrN-0lmQptI/edit?gid=0#gid=0
* engineer plan: https://docs.google.com/document/d/1hgIbQsPX-tQ5CF34h3HKw3hM1S8jAv7i1C7_QIL79-U/edit?tab=t.0

## Feature
upload(file) -> None
download(filename) -> file
list() -> array<filename:string>

## Requirements
* Node discovery using a peer-to-peer networking protocol
**  
* File chunking and distribution
* Data redundancy for reliability
* Basic user authentication and access control
* A front-end user interface that can be as simple as a command-line utility


## Workflow
s0 .. s9
upload:
USEREND
* chunk: splice into small pieces locally
* distributed algorithm(threshold, nums_copy) -> map(server:[file]):
** if file size < threshold: make <nums_copy> copy and randomly select <nums_copy> server as destination
** else: slice smaller piece, (nums_copy*file_size//threhsold) file copy and randomly assign to all servers
* upload(map(server:[file]):

{s1:[a.txt.1.10, ], s2 ... s10}
a.txt -> a.txt.1.10 (1kb).. a.txt.10.10 (1kb)
s0: (a.txt.1.10 a.txt.5.10 a.txt.6.10)
s1: (a.txt.2.10 a.txt.3.10 a.txt.5.10)
s2: (a.txt.1.10 a.txt.4.10 a.txt.9.10)
s3: (a.txt.6.10 a.txt.7.10 a.txt.10.10)
s4: (a.txt.7.10 a.txt.8.10 a.txt.9.10)
SERVER 1
closer_friend []
* 
