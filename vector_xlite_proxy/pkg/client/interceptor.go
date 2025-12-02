package client

import (
	"context"
	"fmt"
	"log"
	"sync"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// RedirectInterceptor handles automatic redirection to leader
type RedirectInterceptor struct {
	maxRedirects  int
	connCache     map[string]*grpc.ClientConn
	connCacheMux  sync.RWMutex
}

// NewRedirectInterceptor creates a new redirect interceptor
func NewRedirectInterceptor(maxRedirects int) *RedirectInterceptor {
	if maxRedirects <= 0 {
		maxRedirects = 3
	}

	return &RedirectInterceptor{
		maxRedirects: maxRedirects,
		connCache:    make(map[string]*grpc.ClientConn),
	}
}

// Unary returns the unary client interceptor
func (i *RedirectInterceptor) Unary() grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		return i.invokeWithRedirect(ctx, method, req, reply, cc, invoker, opts, 0)
	}
}

// invokeWithRedirect recursively handles redirects
func (i *RedirectInterceptor) invokeWithRedirect(
	ctx context.Context,
	method string,
	req, reply interface{},
	cc *grpc.ClientConn,
	invoker grpc.UnaryInvoker,
	opts []grpc.CallOption,
	redirectCount int,
) error {
	// Check redirect limit
	if redirectCount >= i.maxRedirects {
		return fmt.Errorf("max redirects (%d) exceeded", i.maxRedirects)
	}

	// Capture response headers
	var header metadata.MD
	opts = append(opts, grpc.Header(&header))

	// Make the RPC call
	err := invoker(ctx, method, req, reply, cc, opts...)

	// Check if we got a redirect response
	if err != nil {
		st := status.Convert(err)

		// Check if this is a redirect error
		if st.Code() == codes.FailedPrecondition {
			// Extract leader address from metadata
			leaderAddrs := header.Get("x-leader-addr")
			isRedirect := header.Get("x-redirect")

			if len(leaderAddrs) > 0 && len(isRedirect) > 0 && isRedirect[0] == "true" {
				leaderAddr := leaderAddrs[0]
				log.Printf("[Redirect] Redirecting to leader: %s (attempt %d)", leaderAddr, redirectCount+1)

				// Get or create connection to leader
				leaderConn, err := i.getOrCreateConnection(leaderAddr)
				if err != nil {
					return fmt.Errorf("failed to connect to leader %s: %w", leaderAddr, err)
				}

				// Retry the request on the leader
				return i.invokeWithRedirect(
					ctx,
					method,
					req,
					reply,
					leaderConn,
					invoker,
					opts[:len(opts)-1], // Remove the header option to avoid duplicates
					redirectCount+1,
				)
			}
		}
	}

	return err
}

// getOrCreateConnection gets an existing connection or creates a new one
func (i *RedirectInterceptor) getOrCreateConnection(addr string) (*grpc.ClientConn, error) {
	// Check cache first (read lock)
	i.connCacheMux.RLock()
	if conn, exists := i.connCache[addr]; exists {
		i.connCacheMux.RUnlock()
		return conn, nil
	}
	i.connCacheMux.RUnlock()

	// Create new connection (write lock)
	i.connCacheMux.Lock()
	defer i.connCacheMux.Unlock()

	// Double-check after acquiring write lock
	if conn, exists := i.connCache[addr]; exists {
		return conn, nil
	}

	// Create new connection
	conn, err := grpc.Dial(
		addr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		return nil, err
	}

	i.connCache[addr] = conn
	return conn, nil
}

// Close closes all cached connections
func (i *RedirectInterceptor) Close() error {
	i.connCacheMux.Lock()
	defer i.connCacheMux.Unlock()

	var errs []error
	for addr, conn := range i.connCache {
		if err := conn.Close(); err != nil {
			errs = append(errs, fmt.Errorf("failed to close connection to %s: %w", addr, err))
		}
	}

	i.connCache = make(map[string]*grpc.ClientConn)

	if len(errs) > 0 {
		return fmt.Errorf("errors closing connections: %v", errs)
	}

	return nil
}

// LoggingInterceptor logs all outgoing requests
type LoggingInterceptor struct {
	verbose bool
}

// NewLoggingInterceptor creates a new logging interceptor
func NewLoggingInterceptor(verbose bool) *LoggingInterceptor {
	return &LoggingInterceptor{
		verbose: verbose,
	}
}

// Unary returns the unary client interceptor for logging
func (i *LoggingInterceptor) Unary() grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		if i.verbose {
			log.Printf("[Client] Calling method: %s", method)
		}

		err := invoker(ctx, method, req, reply, cc, opts...)

		if err != nil {
			log.Printf("[Client] Method: %s, Error: %v", method, err)
		} else if i.verbose {
			log.Printf("[Client] Method: %s, Success", method)
		}

		return err
	}
}
