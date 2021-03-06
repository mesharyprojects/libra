
<a name="0x1_DualAttestationLimit"></a>

# Module `0x1::DualAttestationLimit`

### Table of Contents

-  [Struct `DualAttestationLimit`](#0x1_DualAttestationLimit_DualAttestationLimit)
-  [Struct `ModifyLimitCapability`](#0x1_DualAttestationLimit_ModifyLimitCapability)
-  [Function `initialize`](#0x1_DualAttestationLimit_initialize)
-  [Function `get_cur_microlibra_limit`](#0x1_DualAttestationLimit_get_cur_microlibra_limit)
-  [Function `set_microlibra_limit`](#0x1_DualAttestationLimit_set_microlibra_limit)



<a name="0x1_DualAttestationLimit_DualAttestationLimit"></a>

## Struct `DualAttestationLimit`



<pre><code><b>struct</b> <a href="#0x1_DualAttestationLimit">DualAttestationLimit</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>micro_lbr_limit: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DualAttestationLimit_ModifyLimitCapability"></a>

## Struct `ModifyLimitCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_DualAttestationLimit_ModifyLimitCapability">ModifyLimitCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;<a href="#0x1_DualAttestationLimit_DualAttestationLimit">DualAttestationLimit::DualAttestationLimit</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DualAttestationLimit_initialize"></a>

## Function `initialize`

Travel rule limit set during genesis


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_initialize">initialize</a>(account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_initialize">initialize</a>(account: &signer, tc_account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>(), 1);
    <b>let</b> cap = <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config_with_capability">LibraConfig::publish_new_config_with_capability</a>&lt;<a href="#0x1_DualAttestationLimit">DualAttestationLimit</a>&gt;(
        account,
        <a href="#0x1_DualAttestationLimit">DualAttestationLimit</a> { micro_lbr_limit: 1000000 },
    );
    move_to(tc_account, <a href="#0x1_DualAttestationLimit_ModifyLimitCapability">ModifyLimitCapability</a> { cap })
}
</code></pre>



</details>

<a name="0x1_DualAttestationLimit_get_cur_microlibra_limit"></a>

## Function `get_cur_microlibra_limit`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_get_cur_microlibra_limit">get_cur_microlibra_limit</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_get_cur_microlibra_limit">get_cur_microlibra_limit</a>(): u64 {
    <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_DualAttestationLimit">DualAttestationLimit</a>&gt;().micro_lbr_limit
}
</code></pre>



</details>

<a name="0x1_DualAttestationLimit_set_microlibra_limit"></a>

## Function `set_microlibra_limit`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_set_microlibra_limit">set_microlibra_limit</a>(tc_account: &signer, micro_lbr_limit: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_DualAttestationLimit_set_microlibra_limit">set_microlibra_limit</a>(tc_account: &signer, micro_lbr_limit: u64) <b>acquires</b> <a href="#0x1_DualAttestationLimit_ModifyLimitCapability">ModifyLimitCapability</a> {
    <b>assert</b>(
        micro_lbr_limit &gt;= 1000,
        4
    );
    <b>let</b> tc_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(tc_account);
    <b>assert</b>(tc_address == <a href="Association.md#0x1_Association_treasury_compliance_account">Association::treasury_compliance_account</a>(), 3);
    <b>let</b> modify_cap = &borrow_global&lt;<a href="#0x1_DualAttestationLimit_ModifyLimitCapability">ModifyLimitCapability</a>&gt;(tc_address).cap;
    <a href="LibraConfig.md#0x1_LibraConfig_set_with_capability">LibraConfig::set_with_capability</a>&lt;<a href="#0x1_DualAttestationLimit">DualAttestationLimit</a>&gt;(
        modify_cap,
        <a href="#0x1_DualAttestationLimit">DualAttestationLimit</a> { micro_lbr_limit },
    );
}
</code></pre>



</details>
