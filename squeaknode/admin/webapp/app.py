# MIT License
#
# Copyright (c) 2020 Jonathan Zernik
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
import logging
import os
import threading
from functools import wraps

from flask import flash
from flask import Flask
from flask import redirect
from flask import render_template
from flask import request
from flask import url_for
from flask_cors import CORS
from flask_login import current_user
from flask_login import login_required
from flask_login import login_user
from flask_login import LoginManager
from flask_login import logout_user
from werkzeug.serving import make_server

from proto import lnd_pb2
from proto import squeak_admin_pb2
from squeaknode.admin.webapp.forms import LoginForm
from squeaknode.admin.webapp.user import User

logger = logging.getLogger(__name__)


def create_app(handler, username, password):
    # create and configure the app
    logger.debug("Starting flask app from directory: {}".format(os.getcwd()))
    app = Flask(
        __name__,
        static_url_path="/",
        static_folder="static/build",
    )
    app.config.from_mapping(
        SECRET_KEY="dev",
    )
    login = LoginManager(app)
    valid_user = User(
        username,
        password,
    )
    logger.debug("Starting flask with app.root_path: {}".format(app.root_path))
    logger.debug("Files in root path: {}".format(os.listdir(app.root_path)))

    @login.user_loader
    def load_user(id):
        return valid_user.get_user_by_username(id)

    @login.unauthorized_handler
    def unauthorized_callback():
        return redirect(url_for("login"))

    def protobuf_serialized(request_message):
        def decorator(func):
            @wraps(func)
            def wrapper(*args, **kwargs):
                data = request.get_data()
                request_message.ParseFromString(data)
                try:
                    reply = func(request_message)
                    return reply.SerializeToString()
                except Exception as e:
                    logger.exception("Error in handle admin web request.")
                    return str(e), 500
            return wrapper
        return decorator

    @app.route("/login", methods=["GET", "POST"])
    def login():
        logger.info("Trying to login")
        if current_user.is_authenticated:
            return redirect(url_for("index"))
        default_username = request.args.get('user')
        form = LoginForm(username=default_username)
        if form.validate_on_submit():
            user = valid_user.get_user_by_username(form.username.data)
            if user is None or not user.check_password(form.password.data):
                flash("Invalid username or password")
                return redirect(url_for("login"))
            login_user(user, remember=form.remember_me.data)
            return redirect(url_for("index"))
        return render_template("login.html", title="Sign In", form=form)

    @app.route("/logout")
    def logout():
        logout_user()
        return redirect(url_for("index"))

    @app.route("/")
    @login_required
    def index():
        logger.info("Getting index route.")
        return app.send_static_file("index.html")

    @app.route("/user")
    @login_required
    def user():
        logger.info("Getting user.")
        return current_user.username

    @app.route("/lndgetinfo", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.GetInfoRequest())
    def lndgetinfo(msg):
        return handler.handle_lnd_get_info(msg)

    @app.route("/lndwalletbalance", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.WalletBalanceRequest())
    def lndwalletbalance(msg):
        return handler.handle_lnd_wallet_balance(msg)

    @app.route("/lndgettransactions", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.GetTransactionsRequest())
    def lndgettransactions(msg):
        return handler.handle_lnd_get_transactions(msg)

    @app.route("/lndlistpeers", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.ListPeersRequest())
    def lndlistpeers(msg):
        return handler.handle_lnd_list_peers(msg)

    @app.route("/lndlistchannels", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.ListChannelsRequest())
    def lndlistchannels(msg):
        return handler.handle_lnd_list_channels(msg)

    @app.route("/lndpendingchannels", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.PendingChannelsRequest())
    def lndpendingchannels(msg):
        return handler.handle_lnd_pending_channels(msg)

    @app.route("/lndconnectpeer", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.ConnectPeerRequest())
    def lndconnectpeer(msg):
        return handler.handle_lnd_connect_peer(msg)

    @app.route("/lnddisconnectpeer", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.DisconnectPeerRequest())
    def lnddisconnectpeer(msg):
        return handler.handle_lnd_disconnect_peer(msg)

    @app.route("/lndopenchannelsync", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.OpenChannelRequest())
    def lndopenchannelsync(msg):
        return handler.handle_lnd_open_channel_sync(msg)

    @app.route("/lndclosechannel", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.CloseChannelRequest())
    def lndclosechannel(msg):
        return handler.handle_lnd_close_channel(msg)

    @app.route("/lndnewaddress", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.NewAddressRequest())
    def lndnewaddress(msg):
        return handler.handle_lnd_new_address(msg)

    @app.route("/lndsendcoins", methods=["POST"])
    @login_required
    @protobuf_serialized(lnd_pb2.SendCoinsRequest())
    def lndsendcoins(msg):
        return handler.handle_lnd_send_coins(msg)

    @app.route("/gettimelinesqueakdisplays", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetTimelineSqueakDisplaysRequest())
    def gettimelinesqueakdisplays(msg):
        return handler.handle_get_timeline_squeak_display_entries(msg)

    @app.route("/getsqueakprofile", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSqueakProfileRequest())
    def getsqueakprofile(msg):
        return handler.handle_get_squeak_profile(msg)

    @app.route("/setsqueakprofilefollowing", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.SetSqueakProfileFollowingRequest())
    def setsqueakprofilefollowing(msg):
        return handler.handle_set_squeak_profile_following(msg)

    @app.route("/renamesqueakprofile", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.RenameSqueakProfileRequest())
    def renamesqueakprofile(msg):
        return handler.handle_rename_squeak_profile(msg)

    @app.route("/setsqueakprofileimage", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.SetSqueakProfileImageRequest())
    def setsqueakprofileimage(msg):
        return handler.handle_set_squeak_profile_image(msg)

    @app.route("/clearsqueakprofileimage", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.ClearSqueakProfileImageRequest())
    def clearsqueakprofileimage(msg):
        return handler.handle_clear_squeak_profile_image(msg)

    @app.route("/getpeers", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetPeersRequest())
    def getpeers(msg):
        return handler.handle_get_squeak_peers(msg)

    @app.route("/payoffer", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.PayOfferRequest())
    def payoffer(msg):
        return handler.handle_pay_offer(msg)

    @app.route("/decryptsqueak", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.DecryptSqueakRequest())
    def decryptsqueak(msg):
        return handler.handle_decrypt_squeak(msg)

    @app.route("/getbuyoffers", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetBuyOffersRequest())
    def getbuyoffers(msg):
        return handler.handle_get_buy_offers(msg)

    @app.route("/getbuyoffer", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetBuyOfferRequest())
    def getbuyoffer(msg):
        return handler.handle_get_buy_offer(msg)

    @app.route("/getpeer", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetPeerRequest())
    def getpeer(msg):
        return handler.handle_get_squeak_peer(msg)

    @app.route("/getpeerbyaddress", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetPeerByAddressRequest())
    def getpeerbyaddress(msg):
        return handler.handle_get_squeak_peer_by_address(msg)

    @app.route("/setpeerautoconnect", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.SetPeerAutoconnectRequest())
    def setpeerautoconnect(msg):
        return handler.handle_set_squeak_peer_autoconnect(msg)

    @app.route("/setpeershareforfree", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.SetPeerShareForFreeRequest())
    def setpeershareforfree(msg):
        return handler.handle_set_squeak_peer_share_for_free(msg)

    @app.route("/renamepeer", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.RenamePeerRequest())
    def renamepeer(msg):
        return handler.handle_rename_squeak_peer(msg)

    @app.route("/getprofiles", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetProfilesRequest())
    def getprofiles(msg):
        return handler.handle_get_profiles(msg)

    @app.route("/getsigningprofiles", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSigningProfilesRequest())
    def getsigningprofiles(msg):
        return handler.handle_get_signing_profiles(msg)

    @app.route("/getcontactprofiles", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetContactProfilesRequest())
    def getcontactprofiles(msg):
        return handler.handle_get_contact_profiles(msg)

    @app.route("/makesqueakrequest", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.MakeSqueakRequest())
    def makesqueakrequest(msg):
        return handler.handle_make_squeak(msg)

    @app.route("/makeresqueak", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.MakeResqueakRequest())
    def makeresqueak(msg):
        return handler.handle_make_resqueak(msg)

    @app.route("/getsqueakdisplay", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSqueakDisplayRequest())
    def getsqueakdisplay(msg):
        return handler.handle_get_squeak_display_entry(msg)

    @app.route("/getancestorsqueakdisplays", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetAncestorSqueakDisplaysRequest())
    def getancestorsqueakdisplays(msg):
        return handler.handle_get_ancestor_squeak_display_entries(msg)

    @app.route("/getreplysqueakdisplays", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetReplySqueakDisplaysRequest())
    def getreplysqueakdisplays(msg):
        return handler.handle_get_reply_squeak_display_entries(msg)

    @app.route("/getsqueakprofilebypubkey", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSqueakProfileByPubKeyRequest())
    def getsqueakprofilebypubkey(msg):
        return handler.handle_get_squeak_profile_by_pubkey(msg)

    @app.route("/getpubkeysqueakdisplays", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetPubKeySqueakDisplaysRequest())
    def getaddresssqueakdisplays(msg):
        return handler.handle_get_squeak_display_entries_for_pubkey(msg)

    @app.route("/getsearchsqueakdisplays", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSearchSqueakDisplaysRequest())
    def getsearchsqueakdisplays(msg):
        return handler.handle_get_squeak_display_entries_for_text_search(msg)

    @app.route("/createcontactprofile", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.CreateContactProfileRequest())
    def createcontactprofile(msg):
        return handler.handle_create_contact_profile(msg)

    @app.route("/createsigningprofile", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.CreateSigningProfileRequest())
    def createsigningprofile(msg):
        return handler.handle_create_signing_profile(msg)

    @app.route("/importsigningprofile", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.ImportSigningProfileRequest())
    def importsigningprofile(msg):
        return handler.handle_import_signing_profile(msg)

    @app.route("/createpeer", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.CreatePeerRequest())
    def createpeer(msg):
        return handler.handle_create_peer(msg)

    @app.route("/deletepeer", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.DeletePeerRequest())
    def deletepeer(msg):
        return handler.handle_delete_squeak_peer(msg)

    @app.route("/deleteprofile", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.DeleteSqueakProfileRequest())
    def deleteprofile(msg):
        return handler.handle_delete_squeak_profile(msg)

    @app.route("/deletesqueak", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.DeleteSqueakRequest())
    def deletesqueak(msg):
        return handler.handle_delete_squeak(msg)

    @app.route("/downloadsqueak", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.DownloadSqueakRequest())
    def downloadsqueak(msg):
        return handler.handle_download_squeak(msg)

    @app.route("/downloadsqueaksecretkey", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.DownloadSqueakSecretKeyRequest())
    def downloadsqueaksecretkey(msg):
        return handler.handle_download_squeak_secret_key(msg)

    @app.route("/downloadoffers", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.DownloadOffersRequest())
    def downloadoffers(msg):
        return handler.handle_download_offers(msg)

    @app.route("/downloadreplies", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.DownloadRepliesRequest())
    def downloadreplies(msg):
        return handler.handle_download_replies(msg)

    @app.route("/downloadaddresssqueaks", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.DownloadPubKeySqueaksRequest())
    def downloadaddresssqueaks(msg):
        return handler.handle_download_pubkey_squeaks(msg)

    @app.route("/getsentpayments", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSentPaymentsRequest())
    def getsentpayments(msg):
        return handler.handle_get_sent_payments(msg)

    @app.route("/getsentpaymentsforsqueak", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSentPaymentsForSqueakRequest())
    def getsentpaymentsforsqueak(msg):
        return handler.handle_get_sent_payments_for_squeak(msg)

    @app.route("/getsentpaymentsforpubkey", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSentPaymentsForPubkeyRequest())
    def getsentpaymentsforpubkey(msg):
        return handler.handle_get_sent_payments_for_pubkey(msg)

    @app.route("/getsentpaymentsforpeer", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSentPaymentsForPeerRequest())
    def getsentpaymentsforpeer(msg):
        return handler.handle_get_sent_payments_for_peer(msg)

    @app.route("/getsentoffers", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSentOffersRequest())
    def getsentoffers(msg):
        return handler.handle_get_sent_offers(msg)

    @app.route("/getreceivedpayments", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetReceivedPaymentsRequest())
    def getreceivedpayments(msg):
        return handler.handle_get_received_payments(msg)

    @app.route("/getreceivedpaymentsforsqueak", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetReceivedPaymentsForSqueakRequest())
    def getreceivedpaymentsforsqueak(msg):
        return handler.handle_get_received_payments_for_squeak(msg)

    @app.route("/getreceivedpaymentsforpubkey", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetReceivedPaymentsForPubkeyRequest())
    def getreceivedpaymentsforpubkey(msg):
        return handler.handle_get_received_payments_for_pubkey(msg)

    @app.route("/getreceivedpaymentsforpeer", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetReceivedPaymentsForPeerRequest())
    def getreceivedpaymentsforpeer(msg):
        return handler.handle_get_received_payments_for_peer(msg)

    @app.route("/getnetwork", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetNetworkRequest())
    def getnetwork(msg):
        return handler.handle_get_network(msg)

    @app.route("/getsqueakprofileprivatekey", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSqueakProfilePrivateKeyRequest())
    def getsqueakprofileprivatekey(msg):
        return handler.handle_get_squeak_profile_private_key(msg)

    @app.route("/getpaymentsummary", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetPaymentSummaryRequest())
    def getpaymentsummary(msg):
        return handler.handle_get_payment_summary(msg)

    @app.route("/getpaymentsummaryforsqueak", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetPaymentSummaryForSqueakRequest())
    def getpaymentsummaryforsqueak(msg):
        return handler.handle_get_payment_summary_for_squeak(msg)

    @app.route("/getpaymentsummaryforpubkey", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetPaymentSummaryForPubkeyRequest())
    def getpaymentsummaryforpubkey(msg):
        return handler.handle_get_payment_summary_for_pubkey(msg)

    @app.route("/getpaymentsummaryforpeer", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetPaymentSummaryForPeerRequest())
    def getpaymentsummaryforpeer(msg):
        return handler.handle_get_payment_summary_for_peer(msg)

    @app.route("/reprocessreceivedpayments", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.ReprocessReceivedPaymentsRequest())
    def reprocessreceivedpayments(msg):
        return handler.handle_reprocess_received_payments(msg)

    @app.route("/likesqueak", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.LikeSqueakRequest())
    def likesqueak(msg):
        return handler.handle_like_squeak(msg)

    @app.route("/unlikesqueak", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.UnlikeSqueakRequest())
    def unlikesqueak(msg):
        return handler.handle_unlike_squeak(msg)

    @app.route("/getlikedsqueakdisplays", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetLikedSqueakDisplaysRequest())
    def getlikedsqueakdisplays(msg):
        return handler.handle_get_liked_squeak_display_entries(msg)

    @app.route("/getexternaladdress", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetExternalAddressRequest())
    def getexternaladdress(msg):
        return handler.handle_get_external_address(msg)

    @app.route("/getdefaultpeerport", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetDefaultPeerPortRequest())
    def getdefaultpeerport(msg):
        return handler.handle_get_default_peer_port(msg)

    @app.route("/setsellprice", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.SetSellPriceRequest())
    def setsellprice(msg):
        return handler.handle_set_sell_price(msg)

    @app.route("/clearsellprice", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.ClearSellPriceRequest())
    def clearsellprice(msg):
        return handler.handle_clear_sell_price(msg)

    @app.route("/getsellprice", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetSellPriceRequest())
    def getsellprice(msg):
        return handler.handle_get_sell_price(msg)

    @app.route("/addtwitteraccount", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.AddTwitterAccountRequest())
    def addtwitteraccount(msg):
        return handler.handle_add_twitter_account(msg)

    @app.route("/gettwitteraccounts", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.GetTwitterAccountsRequest())
    def gettwitteraccounts(msg):
        return handler.handle_get_twitter_accounts(msg)

    @app.route("/deletetwitteraccount", methods=["POST"])
    @login_required
    @protobuf_serialized(squeak_admin_pb2.DeleteTwitterAccountRequest())
    def deletetwitteraccount(msg):
        return handler.handle_delete_twitter_account(msg)

    return app


class SqueakAdminWebServer:
    def __init__(
        self,
        host,
        port,
        username,
        password,
        use_ssl,
        login_disabled,
        allow_cors,
        handler,
    ):
        self.host = host
        self.port = port
        self.use_ssl = use_ssl
        self.login_disabled = login_disabled
        self.allow_cors = allow_cors
        self.app = create_app(handler, username, password)
        self.server = None

    def get_app(self):

        # Set LOGIN_DISABLED and allow CORS if login not required.
        if self.login_disabled:
            self.app.config["LOGIN_DISABLED"] = True

        # Allow CORS
        if self.allow_cors:
            CORS(self.app)

        return self.app

    def start(self):
        self.server = make_server(
            self.host,
            self.port,
            self.get_app(),
            threaded=True,
            ssl_context="adhoc" if self.use_ssl else None,
        )

        logger.info("Starting SqueakAdminWebServer...")
        threading.Thread(
            target=self.server.serve_forever,
        ).start()

    def stop(self):
        if self.server is None:
            return
        logger.info("Stopping SqueakAdminWebServer....")
        self.server.shutdown()
        logger.info("Stopped SqueakAdminWebServer.")
