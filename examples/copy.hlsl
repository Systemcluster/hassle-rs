Texture2D<float4> g_input    : register(t0, space0);
RWTexture2D<float4> g_output : register(u0, space0);

[numthreads(8, 8, 1)]
void copyCs(uint3 dispatchThreadId : SV_DispatchThreadID)
{
	int x = 0;
	if (dispatchThreadId.x == 0) {
		x = g_output[dispatchThreadId.xz] = g_output[dispatchThreadId.yz];
	} else {
		x = g_output[dispatchThreadId.yz] = g_output[dispatchThreadId.xz];
	}
	g_output[dispatchThreadId.xy] = g_input[dispatchThreadId.xy] + x;
}
