References:

- https://www.youtube.com/watch?v=ZOtNXJCEcF8&list=PLJEZDlUEtOf4zr5cBDdt3DP7QLEd55S38&index=6
- rust book https://doc.rust-lang.org/book/ch15-04-rc.html
- chatgpt


create async runtime from scatch instead using lib async runtime (tokio/async-td)
todo:
- change channel approach (making it more efficient) ->channel make it block which not the problem, the expected is to make the runtime block cs main fn should be not asyn
- thread pools
- thread still spawn every task (maybe leave it like that cs this is a demo)
- sleep timer still using sync timer (maybe leave it like that)