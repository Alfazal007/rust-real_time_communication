package main

import (
	"fmt"
	"io"
	"log"
	"net"
	"sync"
)

func main() {
	conn, err := net.Listen("tcp", "127.0.0.1:8002")
	if err != nil {
		log.Fatal("Issue starting the TCP connection")
	}

	var topWaitGroup sync.WaitGroup

	topWaitGroup.Add(1)
	go func() {
		defer topWaitGroup.Done()
		defer conn.Close()
		for {
			fmt.Printf("Awaiting a new connection with number\n")
			client, err := conn.Accept()
			if err != nil {
				// This connection failed try to serve someone else
				log.Fatal("Issue connecting to the server")
				client.Close()
				continue
			}

			server, err := net.Dial("tcp", "127.0.0.1:8000")
			if err != nil {
				// This connection failed try to serve someone else
				log.Fatal("Issue connecting to the server")
				client.Close()
				server.Close()
				continue
			}

			go func() {
				defer client.Close()
				io.Copy(server, client)
			}()

			go func() {
				defer server.Close()
				io.Copy(client, server)
			}()
		}
	}()

	topWaitGroup.Wait()
}
