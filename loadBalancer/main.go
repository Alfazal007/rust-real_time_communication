package main

import (
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"sync"

	"github.com/Alfazal007/rust-real_time_communication.git/algorithms"
	"github.com/joho/godotenv"
)

func main() {
	err := godotenv.Load()
	if err != nil {
		log.Fatal("Error loading .env file")
	}
	conn, err := net.Listen("tcp", "127.0.0.1:8004")
	if err != nil {
		log.Fatal("Issue starting the TCP connection")
	}
	type_of_server := os.Getenv("TYPE")

	var topWaitGroup sync.WaitGroup

	topWaitGroup.Add(1)
	go func() {
		defer topWaitGroup.Done()
		defer conn.Close()
		for {
			fmt.Printf("Awaiting a new connection with number\n")
			client, err := conn.Accept()
			if err != nil {
				fmt.Println("Issue connecting to the client")
				continue
			}

			server_to_connect := algorithms.Round_robin_impl(type_of_server)
			server, err := net.Dial("tcp", server_to_connect)
			if err != nil {
				fmt.Println("Issue connecting to the server")
				client.Close()
				continue
			}

			go func() {
				fmt.Println("Client started listen")
				defer client.Close()
				io.Copy(server, client)
				fmt.Println("Client ended listen")
			}()

			go func() {
				fmt.Println("Server started listen")
				defer client.Close()
				defer server.Close()
				io.Copy(client, server)
				fmt.Println("Server ended listen")
			}()
		}
	}()

	topWaitGroup.Wait()
}
