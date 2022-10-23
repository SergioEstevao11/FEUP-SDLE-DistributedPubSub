# SDLE First Assignment

## SDLE First Assignment of group T03G15;.

### Group members:

1. Carlos Gomes (up201906622@edu.fe.up.pt)
2. José Costa (up201907216@edu.fe.up.pt)
3. Pedro Silva (up201907523@edu.fe.up.pt)
4. Sérgio Estevão (up201905680@edu.fe.up.pt)

## How to run

Having [`rust`](https://www.rust-lang.org/) and [`zmqlib`](https://zeromq.org/download/) installed, and inside the `src` directory, the steps to compile and run the applications are as follows:

- For the server application:

    > cargo run --bin client &lt;IP&gt; <SERVER_IP> <SERVER_PORT>

- For the server application:
    > cargo run --bin server &lt;IP&gt; <BIND_PORT>


## Using the application

### Client operations

- SUB <topic_name>
- UNSUB <topic_name>
- PUT <topic_name> <is_retry> <message> 
- GET <topic_name>

Where:

- <topic_name> is any string
- &lt;message&gt; is any string
- <is_retry> is true | false
