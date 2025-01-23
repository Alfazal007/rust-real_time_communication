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

	var waitGroup sync.WaitGroup
	waitGroup.Add(1)

	go func() {
		defer waitGroup.Done()
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
			var curWg sync.WaitGroup
			curWg.Add(2)
			go func() {
				defer curWg.Done()
				io.Copy(server, client)
			}()
			go func() {
				defer curWg.Done()
				io.Copy(client, server)
			}()
			curWg.Wait()
			fmt.Println("Closing the server connection")
			server.Close()
			fmt.Println("Closing the client connection")
			client.Close()

		}
	}()

	waitGroup.Wait()
	conn.Close()
	fmt.Println("Done")
}
