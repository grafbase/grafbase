package main

import (
	"context"
	"fmt"
	"log"
	"net"
	"os"
	"strings"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// PartData represents local part data
type PartData struct {
	Id                string
	PartNumber        string
	Name              string
	Description       string
	Cost              float64
	Supplier          string
	WarehouseId       string
	QuantityAvailable int32
	Category          string
	IsCritical        bool
}

// Hardcoded parts data with references to warehouse locations
var partsList = []PartData{
	// Mountain bike parts
	{
		Id:                "part-001",
		PartNumber:        "CF-FRAME-001",
		Name:              "Carbon Fiber Frame",
		Description:       "Lightweight carbon fiber mountain bike frame",
		Cost:              899.99,
		Supplier:          "CarbonTech Industries",
		WarehouseId:       "loc-001", // Seattle warehouse
		QuantityAvailable: 25,
		Category:          "Frames",
		IsCritical:        true,
	},
	{
		Id:                "part-002",
		PartNumber:        "WHEEL-26-001",
		Name:              "26\" Mountain Wheel",
		Description:       "Durable mountain bike wheel with reinforced spokes",
		Cost:              149.99,
		Supplier:          "WheelMasters",
		WarehouseId:       "loc-001", // Seattle warehouse
		QuantityAvailable: 50,
		Category:          "Wheels",
		IsCritical:        true,
	},
	{
		Id:                "part-003",
		PartNumber:        "DERAIL-XT-001",
		Name:              "XT Derailleur",
		Description:       "High-performance rear derailleur",
		Cost:              199.99,
		Supplier:          "ShiftGear Corp",
		WarehouseId:       "loc-002", // Portland warehouse
		QuantityAvailable: 30,
		Category:          "Drivetrain",
		IsCritical:        true,
	},
	{
		Id:                "part-004",
		PartNumber:        "HBAR-MTN-001",
		Name:              "Mountain Handlebars",
		Description:       "Wide grip mountain bike handlebars",
		Cost:              79.99,
		Supplier:          "HandleBar Pro",
		WarehouseId:       "loc-001", // Seattle warehouse
		QuantityAvailable: 40,
		Category:          "Handlebars",
		IsCritical:        false,
	},
	{
		Id:                "part-005",
		PartNumber:        "BRAKE-DISC-001",
		Name:              "Hydraulic Disc Brake Set",
		Description:       "High-performance hydraulic disc brakes",
		Cost:              249.99,
		Supplier:          "BrakeForce",
		WarehouseId:       "loc-003", // San Francisco warehouse
		QuantityAvailable: 35,
		Category:          "Brakes",
		IsCritical:        true,
	},
	// E-bike specific parts
	{
		Id:                "part-006",
		PartNumber:        "MOTOR-750W-001",
		Name:              "750W Electric Motor",
		Description:       "Brushless electric bike motor",
		Cost:              599.99,
		Supplier:          "ElectroDrive",
		WarehouseId:       "loc-002", // Portland warehouse
		QuantityAvailable: 15,
		Category:          "Motors",
		IsCritical:        true,
	},
	{
		Id:                "part-007",
		PartNumber:        "BATT-48V-001",
		Name:              "48V Lithium Battery",
		Description:       "High-capacity lithium battery pack",
		Cost:              799.99,
		Supplier:          "PowerCell Tech",
		WarehouseId:       "loc-002", // Portland warehouse
		QuantityAvailable: 20,
		Category:          "Batteries",
		IsCritical:        true,
	},
	{
		Id:                "part-008",
		PartNumber:        "DISP-LCD-001",
		Name:              "LCD Display Unit",
		Description:       "E-bike control display with speedometer",
		Cost:              149.99,
		Supplier:          "DisplayTech",
		WarehouseId:       "loc-003", // San Francisco warehouse
		QuantityAvailable: 25,
		Category:          "Electronics",
		IsCritical:        false,
	},
	// Road bike parts
	{
		Id:                "part-009",
		PartNumber:        "AL-FRAME-001",
		Name:              "Aluminum Road Frame",
		Description:       "Lightweight aluminum road bike frame",
		Cost:              599.99,
		Supplier:          "AluFrame Co",
		WarehouseId:       "loc-001", // Seattle warehouse
		QuantityAvailable: 18,
		Category:          "Frames",
		IsCritical:        true,
	},
	{
		Id:                "part-010",
		PartNumber:        "WHEEL-700C-001",
		Name:              "700C Racing Wheel",
		Description:       "Aerodynamic road bike wheel",
		Cost:              299.99,
		Supplier:          "AeroWheels",
		WarehouseId:       "loc-003", // San Francisco warehouse
		QuantityAvailable: 30,
		Category:          "Wheels",
		IsCritical:        true,
	},
	{
		Id:                "part-011",
		PartNumber:        "HBAR-DROP-001",
		Name:              "Carbon Drop Handlebars",
		Description:       "Lightweight carbon fiber drop handlebars",
		Cost:              199.99,
		Supplier:          "CarbonTech Industries",
		WarehouseId:       "loc-001", // Seattle warehouse
		QuantityAvailable: 22,
		Category:          "Handlebars",
		IsCritical:        false,
	},
}

type partsServer struct {
	UnimplementedPartServiceServer
}

func (s *partsServer) GetPart(ctx context.Context, req *GetPartRequest) (*GetPartResponse, error) {
	for _, part := range partsList {
		if part.Id == req.Id {
			return &GetPartResponse{
				Part: &Part{
					Id:                part.Id,
					PartNumber:        part.PartNumber,
					Name:              part.Name,
					Description:       part.Description,
					Cost:              part.Cost,
					Supplier:          part.Supplier,
					WarehouseId:       part.WarehouseId,
					QuantityAvailable: part.QuantityAvailable,
					Category:          part.Category,
					IsCritical:        part.IsCritical,
				},
			}, nil
		}
	}
	return nil, status.Errorf(codes.NotFound, "Part with id %s not found", req.Id)
}

func (s *partsServer) BatchGetParts(ctx context.Context, req *BatchGetPartsRequest) (*BatchGetPartsResponse, error) {
	var responseParts []*Part

	for _, id := range req.Ids {
		for _, part := range partsList {
			if part.Id == id {
				responseParts = append(responseParts, &Part{
					Id:                part.Id,
					PartNumber:        part.PartNumber,
					Name:              part.Name,
					Description:       part.Description,
					Cost:              part.Cost,
					Supplier:          part.Supplier,
					WarehouseId:       part.WarehouseId,
					QuantityAvailable: part.QuantityAvailable,
					Category:          part.Category,
					IsCritical:        part.IsCritical,
				})
				break
			}
		}
	}

	return &BatchGetPartsResponse{Parts: responseParts}, nil
}

func (s *partsServer) SearchParts(ctx context.Context, req *SearchPartsRequest) (*SearchPartsResponse, error) {
	var filteredParts []*Part

	for _, part := range partsList {
		// Start with all parts, then apply filters
		match := true

		// Filter by name (partial match)
		if req.Name != "" && !strings.Contains(strings.ToLower(part.Name), strings.ToLower(req.Name)) {
			match = false
		}

		// Filter by part number (partial match)
		if match && req.PartNumber != "" && !strings.Contains(strings.ToLower(part.PartNumber), strings.ToLower(req.PartNumber)) {
			match = false
		}

		// Filter by category
		if match && req.Category != "" && strings.ToLower(part.Category) != strings.ToLower(req.Category) {
			match = false
		}

		// Filter by supplier
		if match && req.Supplier != "" && strings.ToLower(part.Supplier) != strings.ToLower(req.Supplier) {
			match = false
		}

		// Filter by warehouse ID
		if match && req.WarehouseId != "" && part.WarehouseId != req.WarehouseId {
			match = false
		}

		// Filter by minimum quantity available
		if match && req.MinQuantity > 0 && part.QuantityAvailable < req.MinQuantity {
			match = false
		}

		// Filter by is_critical flag (only filter if explicitly set in request)
		if match && req.IsCritical && !part.IsCritical {
			match = false
		}

		// Filter by minimum cost
		if match && req.MinCost > 0 && part.Cost < req.MinCost {
			match = false
		}

		// Filter by maximum cost
		if match && req.MaxCost > 0 && part.Cost > req.MaxCost {
			match = false
		}

		if match {
			filteredParts = append(filteredParts, &Part{
				Id:                part.Id,
				PartNumber:        part.PartNumber,
				Name:              part.Name,
				Description:       part.Description,
				Cost:              part.Cost,
				Supplier:          part.Supplier,
				WarehouseId:       part.WarehouseId,
				QuantityAvailable: part.QuantityAvailable,
				Category:          part.Category,
				IsCritical:        part.IsCritical,
			})
		}
	}

	return &SearchPartsResponse{Parts: filteredParts}, nil
}

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "50052"
	}

	lis, err := net.Listen("tcp", ":"+port)
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}

	s := grpc.NewServer()
	RegisterPartServiceServer(s, &partsServer{})

	fmt.Printf("Parts service (Go) running on port %s\n", port)
	if err := s.Serve(lis); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
}
