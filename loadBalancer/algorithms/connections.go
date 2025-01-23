package algorithms

func server_connections_array() []string {
	addresses := make([]string, 0)
	addresses = append(addresses, "127.0.0.1:8000")
	addresses = append(addresses, "127.0.0.1:8001")
	return addresses
}

func socket_connections_array() []string {
	addresses := make([]string, 0)
	addresses = append(addresses, "127.0.0.1:8002")
	addresses = append(addresses, "127.0.0.1:8003")
	return addresses
}
