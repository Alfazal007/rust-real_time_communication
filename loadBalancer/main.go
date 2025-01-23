package main

import (
	"fmt"
	"log"
	"net"
)

func main() {
	conn, err := net.Listen("tcp", "127.0.0.1:8002")
	if err != nil {
		log.Fatal("Issue starting the TCP connection")
	}

	connNumber := 1
	for {
		fmt.Printf("Awaiting a new connection with number %v\n", connNumber)
		client, err := conn.Accept()
		if err != nil {
			log.Fatal("Issue connecting to the server")
		}
		var buf []byte
		client.Read(buf)
		fmt.Println("The data from client is ", string(buf))
		_, err = client.Write([]byte("Hello client from server"))
		fmt.Println(err)
		client.Close()
		connNumber += 1
	}
}
