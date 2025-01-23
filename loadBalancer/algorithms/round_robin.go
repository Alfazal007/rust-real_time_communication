package algorithms

var address_index = 0

func Round_robin_impl(type_of_server string) string {
	var address_to_return string
	if type_of_server == "websocket" {
		address_to_return = socket_connections_array()[address_index]
	} else {
		address_to_return = server_connections_array()[address_index]
	}
	if address_index == 0 {
		address_index = 1
	} else {
		address_index = 0
	}
	return address_to_return
}
